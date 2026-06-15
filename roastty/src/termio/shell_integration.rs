//! Automatic shell-integration setup.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use crate::config::{ShellIntegration, ShellIntegrationFeatures};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Command {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) env: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    Zsh,
}

pub(crate) fn setup_features(
    env: &mut Vec<(String, String)>,
    features: ShellIntegrationFeatures,
    cursor_blink: bool,
) {
    let mut parts = Vec::new();
    if features.cursor {
        parts.push(if cursor_blink {
            "cursor:blink".to_string()
        } else {
            "cursor:steady".to_string()
        });
    }
    if features.path {
        parts.push("path".to_string());
    }
    if features.ssh_env {
        parts.push("ssh-env".to_string());
    }
    if features.ssh_terminfo {
        parts.push("ssh-terminfo".to_string());
    }
    if features.sudo {
        parts.push("sudo".to_string());
    }
    if features.title {
        parts.push("title".to_string());
    }

    if parts.is_empty() {
        remove_env(env, "ROASTTY_SHELL_FEATURES");
    } else {
        put_env(env, "ROASTTY_SHELL_FEATURES", parts.join(","));
    }
}

pub(crate) fn setup(
    command: Command,
    resource_dir: &Path,
    integration: ShellIntegration,
) -> Command {
    let shell = match forced_shell(integration).or_else(|| detect_shell(&command.program)) {
        Some(shell) => shell,
        None => return command,
    };

    let mut updated = command.clone();
    let ok = match shell {
        Shell::Bash => setup_bash(&mut updated, resource_dir),
        Shell::Elvish | Shell::Fish => setup_xdg_data_dirs(&mut updated.env, resource_dir),
        Shell::Nushell => setup_nushell(&mut updated, resource_dir),
        Shell::Zsh => setup_zsh(&mut updated, resource_dir),
    };

    if ok {
        updated
    } else {
        command
    }
}

fn forced_shell(integration: ShellIntegration) -> Option<Shell> {
    match integration {
        ShellIntegration::None | ShellIntegration::Detect => None,
        ShellIntegration::Bash => Some(Shell::Bash),
        ShellIntegration::Elvish => Some(Shell::Elvish),
        ShellIntegration::Fish => Some(Shell::Fish),
        ShellIntegration::Nushell => Some(Shell::Nushell),
        ShellIntegration::Zsh => Some(Shell::Zsh),
    }
}

fn detect_shell(program: &OsStr) -> Option<Shell> {
    let exe = Path::new(program).file_name()?.to_str()?;
    match exe {
        "bash" => {
            if cfg!(target_os = "macos") && program == OsStr::new("/bin/bash") {
                None
            } else {
                Some(Shell::Bash)
            }
        }
        "elvish" => Some(Shell::Elvish),
        "fish" => Some(Shell::Fish),
        "nu" => Some(Shell::Nushell),
        "zsh" => Some(Shell::Zsh),
        _ => None,
    }
}

fn setup_bash(command: &mut Command, resource_dir: &Path) -> bool {
    let script_path = resource_dir
        .join("shell-integration")
        .join("bash")
        .join("roastty.bash");
    if !script_path.is_file() {
        return false;
    }

    let mut args = Vec::with_capacity(command.args.len() + 1);
    args.push(OsString::from("--posix"));
    let mut inject = String::from("1");
    let mut rcfile = None;
    let mut iter = command.args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--posix" {
            return false;
        } else if arg == "--norc" {
            inject.push_str(" --norc");
        } else if arg == "--noprofile" {
            inject.push_str(" --noprofile");
        } else if arg == "--rcfile" || arg == "--init-file" {
            rcfile = iter
                .next()
                .map(|value| value.to_string_lossy().into_owned());
        } else if short_option_contains(arg, 'c') {
            return false;
        } else {
            args.push(arg.clone());
            if arg == "-" || arg == "--" {
                args.extend(iter.cloned());
                break;
            }
        }
    }

    if let Some(value) = get_env(&command.env, "ENV").map(str::to_owned) {
        put_env(&mut command.env, "ROASTTY_BASH_ENV", value);
    }
    put_env(
        &mut command.env,
        "ENV",
        script_path.to_string_lossy().into_owned(),
    );
    put_env(&mut command.env, "ROASTTY_BASH_INJECT", inject);
    if let Some(rcfile) = rcfile {
        put_env(&mut command.env, "ROASTTY_BASH_RCFILE", rcfile);
    }
    if get_env(&command.env, "HISTFILE").is_none() {
        if let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) {
            let histfile = PathBuf::from(home).join(".bash_history");
            put_env(
                &mut command.env,
                "HISTFILE",
                histfile.to_string_lossy().into_owned(),
            );
            put_env(&mut command.env, "ROASTTY_BASH_UNEXPORT_HISTFILE", "1");
        }
    }
    command.args = args;
    true
}

fn setup_xdg_data_dirs(env: &mut Vec<(String, String)>, resource_dir: &Path) -> bool {
    let path = resource_dir.join("shell-integration");
    if !path.is_dir() {
        return false;
    }
    let path = path.to_string_lossy().into_owned();
    let old = get_env(env, "XDG_DATA_DIRS").unwrap_or("/usr/local/share:/usr/share");
    let new_value = if old.is_empty() {
        path.clone()
    } else {
        format!("{path}:{old}")
    };
    put_env(env, "ROASTTY_SHELL_INTEGRATION_XDG_DIR", path);
    put_env(env, "XDG_DATA_DIRS", new_value);
    true
}

fn setup_nushell(command: &mut Command, resource_dir: &Path) -> bool {
    if !setup_xdg_data_dirs(&mut command.env, resource_dir) {
        return false;
    }

    let mut args = Vec::with_capacity(command.args.len() + 2);
    args.push(OsString::from("--execute"));
    args.push(OsString::from("use roastty *"));
    let mut iter = command.args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--command" || arg == "--lsp" || short_option_contains(arg, 'c') {
            return true;
        }
        args.push(arg.clone());
        if arg == "-" || arg == "--" {
            args.extend(iter.cloned());
            break;
        }
    }
    command.args = args;
    true
}

fn setup_zsh(command: &mut Command, resource_dir: &Path) -> bool {
    let path = resource_dir.join("shell-integration").join("zsh");
    if !path.is_dir() {
        return false;
    }
    if let Some(value) = get_env(&command.env, "ZDOTDIR").map(str::to_owned) {
        put_env(&mut command.env, "ROASTTY_ZSH_ZDOTDIR", value);
    }
    put_env(
        &mut command.env,
        "ZDOTDIR",
        path.to_string_lossy().into_owned(),
    );
    true
}

fn short_option_contains(arg: &OsStr, option: char) -> bool {
    let Some(arg) = arg.to_str() else {
        return false;
    };
    let mut chars = arg.chars();
    matches!(chars.next(), Some('-'))
        && !matches!(chars.clone().next(), Some('-') | None)
        && chars.any(|ch| ch == option)
}

fn get_env<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .rev()
        .find(|(existing, _)| existing == key)
        .map(|(_, value)| value.as_str())
}

fn put_env(env: &mut Vec<(String, String)>, key: &str, value: impl Into<String>) {
    let value = value.into();
    if let Some((_, existing)) = env.iter_mut().rev().find(|(existing, _)| existing == key) {
        *existing = value;
    } else {
        env.push((key.to_string(), value));
    }
}

fn remove_env(env: &mut Vec<(String, String)>, key: &str) {
    env.retain(|(existing, _)| existing != key);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ShellIntegrationFeatures;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempResources {
        path: PathBuf,
    }

    impl TempResources {
        fn new(shell: Shell) -> Self {
            let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "roastty-shell-integration-{}-{counter}",
                std::process::id()
            ));
            let shell_dir = path.join("shell-integration");
            match shell {
                Shell::Bash => {
                    fs::create_dir_all(shell_dir.join("bash")).expect("create bash resources");
                    fs::write(shell_dir.join("bash/roastty.bash"), b"").expect("write bash");
                }
                Shell::Elvish => {
                    fs::create_dir_all(shell_dir.join("elvish/lib"))
                        .expect("create elvish resources");
                }
                Shell::Fish => {
                    fs::create_dir_all(shell_dir.join("fish/vendor_conf.d"))
                        .expect("create fish resources");
                }
                Shell::Nushell => {
                    fs::create_dir_all(shell_dir.join("nushell/vendor/autoload"))
                        .expect("create nushell resources");
                }
                Shell::Zsh => {
                    fs::create_dir_all(shell_dir.join("zsh")).expect("create zsh resources");
                    fs::write(shell_dir.join("zsh/.zshenv"), b"").expect("write zshenv");
                }
            }
            Self { path }
        }
    }

    impl Drop for TempResources {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn command(program: &str, args: &[&str]) -> Command {
        Command {
            program: OsString::from(program),
            args: args.iter().map(OsString::from).collect(),
            env: Vec::new(),
        }
    }

    #[test]
    fn detect_shell_matches_supported_programs() {
        assert_eq!(detect_shell(OsStr::new("sh")), None);
        assert_eq!(detect_shell(OsStr::new("bash")), Some(Shell::Bash));
        assert_eq!(detect_shell(OsStr::new("elvish")), Some(Shell::Elvish));
        assert_eq!(detect_shell(OsStr::new("fish")), Some(Shell::Fish));
        assert_eq!(detect_shell(OsStr::new("nu")), Some(Shell::Nushell));
        assert_eq!(detect_shell(OsStr::new("zsh")), Some(Shell::Zsh));

        if cfg!(target_os = "macos") {
            assert_eq!(detect_shell(OsStr::new("/bin/bash")), None);
        }

        assert_eq!(
            detect_shell(OsStr::new("/opt/homebrew/bin/bash")),
            Some(Shell::Bash)
        );
    }

    #[test]
    fn force_shell_overrides_detection_for_all_supported_shells() {
        for (shell, integration) in [
            (Shell::Bash, ShellIntegration::Bash),
            (Shell::Elvish, ShellIntegration::Elvish),
            (Shell::Fish, ShellIntegration::Fish),
            (Shell::Nushell, ShellIntegration::Nushell),
            (Shell::Zsh, ShellIntegration::Zsh),
        ] {
            let resources = TempResources::new(shell);
            let cmd = setup(command("sh", &[]), &resources.path, integration);

            match shell {
                Shell::Bash => {
                    assert_eq!(cmd.program, OsString::from("sh"));
                    assert_eq!(cmd.args, [OsString::from("--posix")]);
                    assert!(get_env(&cmd.env, "ENV").is_some());
                }
                Shell::Elvish | Shell::Fish => {
                    assert!(get_env(&cmd.env, "ROASTTY_SHELL_INTEGRATION_XDG_DIR").is_some());
                    assert!(get_env(&cmd.env, "XDG_DATA_DIRS").is_some());
                }
                Shell::Nushell => {
                    assert_eq!(
                        cmd.args,
                        [OsString::from("--execute"), OsString::from("use roastty *")]
                    );
                }
                Shell::Zsh => {
                    assert!(get_env(&cmd.env, "ZDOTDIR").is_some());
                }
            }
        }
    }

    #[test]
    fn features_are_sorted_and_include_cursor_mode() {
        let mut env = Vec::new();
        setup_features(
            &mut env,
            ShellIntegrationFeatures {
                cursor: true,
                sudo: true,
                title: true,
                ssh_env: true,
                ssh_terminfo: true,
                path: true,
            },
            true,
        );
        assert_eq!(
            get_env(&env, "ROASTTY_SHELL_FEATURES"),
            Some("cursor:blink,path,ssh-env,ssh-terminfo,sudo,title")
        );

        setup_features(
            &mut env,
            ShellIntegrationFeatures {
                cursor: true,
                sudo: false,
                title: false,
                ssh_env: false,
                ssh_terminfo: false,
                path: false,
            },
            false,
        );
        assert_eq!(
            get_env(&env, "ROASTTY_SHELL_FEATURES"),
            Some("cursor:steady")
        );
    }

    #[test]
    fn disabled_features_remove_feature_env() {
        let mut env = vec![("ROASTTY_SHELL_FEATURES".to_string(), "old".to_string())];
        setup_features(
            &mut env,
            ShellIntegrationFeatures {
                cursor: false,
                sudo: false,
                title: false,
                ssh_env: false,
                ssh_terminfo: false,
                path: false,
            },
            true,
        );
        assert_eq!(get_env(&env, "ROASTTY_SHELL_FEATURES"), None);
    }

    #[test]
    fn bash_setup_rewrites_args_and_env() {
        let resources = TempResources::new(Shell::Bash);
        let mut cmd = command("bash", &["--norc", "--rcfile", "profile.sh"]);
        cmd.env.push(("ENV".to_string(), "old-env".to_string()));

        let cmd = setup(cmd, &resources.path, ShellIntegration::Detect);

        assert_eq!(cmd.args, [OsString::from("--posix")]);
        assert_eq!(get_env(&cmd.env, "ROASTTY_BASH_ENV"), Some("old-env"));
        assert_eq!(get_env(&cmd.env, "ROASTTY_BASH_INJECT"), Some("1 --norc"));
        assert_eq!(get_env(&cmd.env, "ROASTTY_BASH_RCFILE"), Some("profile.sh"));
        assert!(get_env(&cmd.env, "ENV")
            .expect("ENV")
            .ends_with("shell-integration/bash/roastty.bash"));
    }

    #[test]
    fn bash_unsupported_options_fall_back() {
        let resources = TempResources::new(Shell::Bash);
        for args in [
            &["--posix"][..],
            &["--rcfile", "profile.sh", "--posix"][..],
            &["--init-file", "profile.sh", "--posix"][..],
            &["-c", "echo nope"][..],
            &["-ic", "echo nope"][..],
        ] {
            let original = command("bash", args);
            assert_eq!(
                setup(original.clone(), &resources.path, ShellIntegration::Detect),
                original
            );
        }
    }

    #[test]
    fn bash_setup_inject_flags_rcfiles_history_env_and_separators() {
        let resources = TempResources::new(Shell::Bash);

        let cmd = setup(
            command("bash", &["--noprofile", "--init-file", "profile.sh"]),
            &resources.path,
            ShellIntegration::Detect,
        );
        assert_eq!(cmd.args, [OsString::from("--posix")]);
        assert_eq!(
            get_env(&cmd.env, "ROASTTY_BASH_INJECT"),
            Some("1 --noprofile")
        );
        assert_eq!(get_env(&cmd.env, "ROASTTY_BASH_RCFILE"), Some("profile.sh"));

        let mut with_env = command("bash", &[]);
        with_env.env.push(("ENV".to_string(), "env.sh".to_string()));
        let with_env = setup(with_env, &resources.path, ShellIntegration::Detect);
        assert_eq!(get_env(&with_env.env, "ROASTTY_BASH_ENV"), Some("env.sh"));
        assert!(get_env(&with_env.env, "ENV")
            .expect("ENV")
            .ends_with("shell-integration/bash/roastty.bash"));

        let hist_unset = setup(
            command("bash", &[]),
            &resources.path,
            ShellIntegration::Detect,
        );
        assert!(get_env(&hist_unset.env, "HISTFILE")
            .expect("HISTFILE")
            .ends_with(".bash_history"));
        assert_eq!(
            get_env(&hist_unset.env, "ROASTTY_BASH_UNEXPORT_HISTFILE"),
            Some("1")
        );

        let mut hist_set = command("bash", &[]);
        hist_set
            .env
            .push(("HISTFILE".to_string(), "my_history".to_string()));
        let hist_set = setup(hist_set, &resources.path, ShellIntegration::Detect);
        assert_eq!(get_env(&hist_set.env, "HISTFILE"), Some("my_history"));
        assert_eq!(
            get_env(&hist_set.env, "ROASTTY_BASH_UNEXPORT_HISTFILE"),
            None
        );

        let dash = setup(
            command("bash", &["-", "--arg", "file1", "file2"]),
            &resources.path,
            ShellIntegration::Detect,
        );
        assert_eq!(
            dash.args,
            [
                OsString::from("--posix"),
                OsString::from("-"),
                OsString::from("--arg"),
                OsString::from("file1"),
                OsString::from("file2"),
            ]
        );

        let dashdash = setup(
            command("bash", &["--", "--arg", "file1", "file2"]),
            &resources.path,
            ShellIntegration::Detect,
        );
        assert_eq!(
            dashdash.args,
            [
                OsString::from("--posix"),
                OsString::from("--"),
                OsString::from("--arg"),
                OsString::from("file1"),
                OsString::from("file2"),
            ]
        );
    }

    #[test]
    fn bash_setup_missing_resources_falls_back_without_env_changes() {
        let temp = std::env::temp_dir().join(format!(
            "roastty-shell-integration-bash-missing-{}",
            std::process::id()
        ));
        let original = command("bash", &[]);

        assert_eq!(
            setup(original.clone(), &temp, ShellIntegration::Detect),
            original
        );
    }

    #[test]
    fn zsh_setup_preserves_zdotdir() {
        let resources = TempResources::new(Shell::Zsh);
        let plain = setup(
            command("zsh", &[]),
            &resources.path,
            ShellIntegration::Detect,
        );

        assert!(get_env(&plain.env, "ZDOTDIR")
            .expect("ZDOTDIR")
            .ends_with("shell-integration/zsh"));
        assert_eq!(get_env(&plain.env, "ROASTTY_ZSH_ZDOTDIR"), None);

        let mut cmd = command("zsh", &[]);
        cmd.env
            .push(("ZDOTDIR".to_string(), "/old/zdotdir".to_string()));

        let cmd = setup(cmd, &resources.path, ShellIntegration::Detect);

        assert_eq!(
            get_env(&cmd.env, "ROASTTY_ZSH_ZDOTDIR"),
            Some("/old/zdotdir")
        );
        assert!(get_env(&cmd.env, "ZDOTDIR")
            .expect("ZDOTDIR")
            .ends_with("shell-integration/zsh"));
    }

    #[test]
    fn xdg_setup_prepends_data_dirs() {
        let resources = TempResources::new(Shell::Fish);
        let mut cmd = command("fish", &[]);
        cmd.env
            .push(("XDG_DATA_DIRS".to_string(), "/opt/share".to_string()));

        let cmd = setup(cmd, &resources.path, ShellIntegration::Detect);

        assert!(get_env(&cmd.env, "ROASTTY_SHELL_INTEGRATION_XDG_DIR")
            .expect("xdg dir")
            .ends_with("shell-integration"));
        assert!(get_env(&cmd.env, "XDG_DATA_DIRS")
            .expect("data dirs")
            .ends_with("shell-integration:/opt/share"));
    }

    #[test]
    fn xdg_setup_uses_freedesktop_default_when_unset() {
        let resources = TempResources::new(Shell::Elvish);
        let cmd = setup(
            command("elvish", &[]),
            &resources.path,
            ShellIntegration::Detect,
        );

        assert!(get_env(&cmd.env, "XDG_DATA_DIRS")
            .expect("data dirs")
            .ends_with("shell-integration:/usr/local/share:/usr/share"));
    }

    #[test]
    fn xdg_setup_missing_resources_falls_back_without_env_changes() {
        let temp = std::env::temp_dir().join(format!(
            "roastty-shell-integration-xdg-missing-{}",
            std::process::id()
        ));
        let original = command("fish", &[]);

        assert_eq!(
            setup(original.clone(), &temp, ShellIntegration::Detect),
            original
        );
    }

    #[test]
    fn nushell_setup_adds_execute_use() {
        let resources = TempResources::new(Shell::Nushell);
        let cmd = setup(
            command("nu", &["--login"]),
            &resources.path,
            ShellIntegration::Detect,
        );

        assert_eq!(
            cmd.args,
            [
                OsString::from("--execute"),
                OsString::from("use roastty *"),
                OsString::from("--login")
            ]
        );
        assert!(get_env(&cmd.env, "XDG_DATA_DIRS").is_some());
    }

    #[test]
    fn nushell_unsupported_options_keep_xdg_env_without_command_rewrite() {
        let resources = TempResources::new(Shell::Nushell);
        for args in [
            &["--command", "exit"][..],
            &["--lsp"][..],
            &["-c", "exit"][..],
            &["-ic", "exit"][..],
        ] {
            let cmd = setup(
                command("nu", args),
                &resources.path,
                ShellIntegration::Detect,
            );

            assert_eq!(
                cmd.args,
                args.iter().map(OsString::from).collect::<Vec<_>>(),
                "unsupported args should not be rewritten"
            );
            assert!(get_env(&cmd.env, "ROASTTY_SHELL_INTEGRATION_XDG_DIR")
                .expect("xdg dir")
                .ends_with("shell-integration"));
            assert!(get_env(&cmd.env, "XDG_DATA_DIRS")
                .expect("data dirs")
                .ends_with("shell-integration:/usr/local/share:/usr/share"));
        }
    }

    #[test]
    fn nushell_setup_missing_resources_falls_back_without_env_changes() {
        let temp = std::env::temp_dir().join(format!(
            "roastty-shell-integration-nu-missing-{}",
            std::process::id()
        ));
        let original = command("nu", &[]);

        assert_eq!(
            setup(original.clone(), &temp, ShellIntegration::Detect),
            original
        );
    }

    #[test]
    fn missing_resources_fall_back() {
        let temp = std::env::temp_dir().join(format!(
            "roastty-shell-integration-missing-{}",
            std::process::id()
        ));
        let original = command("zsh", &[]);
        assert_eq!(
            setup(original.clone(), &temp, ShellIntegration::Detect),
            original
        );
    }
}

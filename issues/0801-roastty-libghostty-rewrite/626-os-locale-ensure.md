+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 626: os locale ensureLocale core

## Description

Port the process-locale portion of upstream `os/locale.zig` into
`roastty/src/os/locale.rs`: pre-populate macOS locale env values when needed,
call C `setlocale(LC_ALL, "")`, recover from invalid `LANG`, and fall back to
`en_US.UTF-8`.

This experiment builds on Exp 625's Cocoa probes. Because environment variables
and C locale are process-global, the implementation should separate the
decision/recovery sequence from the real OS calls:

- `ensure_locale()` is the real library helper. It uses `std::env`,
  `macos_lang_from_cocoa`, `macos_language_from_cocoa`, and `libc::setlocale`.
- `ensure_locale_with(...)` is a deterministic test helper. It receives fake env
  accessors, fake Cocoa probe values, and a fake `setlocale` callback so the
  upstream fallback sequence can be tested without mutating the real process
  locale.

## Upstream behavior (`os/locale.zig`)

```zig
pub fn ensureLocale(alloc: std.mem.Allocator) !void {
    const lang = try internal_os.getenv(alloc, "LANG");

    if (comptime builtin.target.os.tag.isDarwin()) {
        if (lang == null or lang.?.value.len == 0) {
            setLangFromCocoa();
        }
    }

    if (setlocale(LC_ALL, "")) |v| return;

    if ((try internal_os.getenv(alloc, "LANG"))) |old_lang| {
        if (old_lang.value.len > 0) {
            _ = internal_os.setenv("LANG", "");
            _ = internal_os.unsetenv("LANG");

            if (setlocale(LC_ALL, "")) |v| {
                if (!std.mem.eql(u8, std.mem.sliceTo(v, 0), "C")) return;
            }
        }
    }

    if (setlocale(LC_ALL, "en_US.UTF-8")) |v| {
        _ = internal_os.setenv("LANG", "en_US.UTF-8");
        return;
    }
}
```

## Rust mapping (`roastty/src/os/locale.rs`)

```rust
pub(crate) fn ensure_locale() -> EnsureLocaleOutcome {
    let mut env = RealLocaleEnv;
    ensure_locale_with(
        &mut env,
        macos_lang_from_cocoa,
        macos_language_from_cocoa,
        real_setlocale,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EnsureLocaleOutcome {
    FromEnvironment(String),
    SystemDefault(String),
    Fallback(String),
    Failed,
}

trait LocaleEnv {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
    fn unset(&mut self, key: &str);
}

fn ensure_locale_with<E, LangProbe, LanguageProbe, SetLocale>(
    env: &mut E,
    lang_probe: LangProbe,
    language_probe: LanguageProbe,
    mut setlocale: SetLocale,
) -> EnsureLocaleOutcome
where
    E: LocaleEnv,
    LangProbe: FnOnce() -> Option<String>,
    LanguageProbe: FnOnce() -> Option<String>,
    SetLocale: FnMut(Option<&str>) -> Option<String>,
{
    if env.get("LANG").as_deref().unwrap_or("").is_empty() {
        if let Some(lang) = lang_probe() {
            env.set("LANG", &lang);
            if let Some(language) = language_probe() {
                env.set("LANGUAGE", &language);
            }
        }
    }

    if let Some(value) = setlocale(Some("")) {
        return EnsureLocaleOutcome::FromEnvironment(value);
    }

    if env.get("LANG").as_deref().unwrap_or("").is_empty() == false {
        env.set("LANG", "");
        env.unset("LANG");
        if let Some(value) = setlocale(Some("")) {
            if value != "C" {
                return EnsureLocaleOutcome::SystemDefault(value);
            }
        }
    }

    if let Some(value) = setlocale(Some("en_US.UTF-8")) {
        env.set("LANG", "en_US.UTF-8");
        return EnsureLocaleOutcome::Fallback(value);
    }

    EnsureLocaleOutcome::Failed
}
```

`real_setlocale(locale)` should convert `None` to a null pointer for future
query use and `Some(value)` to a `CString`, call
`unsafe { libc::setlocale(libc::LC_ALL, ptr) }`, and copy the returned C string
into an owned `String`.

### Notes / deviations

- Upstream returns `void` and logs. Returning `EnsureLocaleOutcome` gives
  Roastty tests and callers observable evidence of which upstream branch won
  without changing the locale decisions.
- The real wrapper may mutate process env and locale; tests should avoid calling
  it except for a conservative smoke check if needed. The branch behavior is
  proven with `ensure_locale_with` fakes.
- The prepopulation step probes `LANGUAGE` only after the Cocoa `LANG` probe
  succeeds, matching upstream `setLangFromCocoa` returning early when
  `languageCode` or `countryCode` is missing.
- The invalid-`LANG` recovery explicitly sets `LANG` to `""` and then unsets it,
  matching upstream's belt-and-suspenders behavior before retrying
  `setlocale(LC_ALL, "")`.
- If the system-default retry returns `"C"`, the implementation does not accept
  it and continues to `en_US.UTF-8`, matching upstream.
- Non-macOS hosts get no Cocoa values because the Exp 625 probes return `None`,
  but the same `setlocale` fallback sequence remains available for host tests.

## Changes

- `roastty/src/os/locale.rs` — add `ensure_locale`, `EnsureLocaleOutcome`,
  `LocaleEnv`, real env adapter, real `setlocale` wrapper, and fake-driven
  tests.

## Verification

- `cargo test -p roastty os::locale::tests` — new tests cover:
  - missing/empty `LANG` prepopulates `LANG` and `LANGUAGE` from probes before
    the first `setlocale("")`;
  - when the Cocoa `LANG` probe returns `None`, `LANGUAGE` is not probed or set;
  - existing non-empty `LANG` skips Cocoa probes;
  - first successful `setlocale("")` returns `FromEnvironment`;
  - failed first `setlocale("")` with non-empty `LANG` clears and unsets `LANG`,
    then accepts a non-`C` system-default result as `SystemDefault`;
  - a system-default retry returning `"C"` is rejected and falls through to
    fallback;
  - fallback success sets `LANG=en_US.UTF-8` and returns `Fallback`;
  - total failure returns `Failed`;
  - `real_setlocale(None)` query smoke test returns a non-empty string without
    mutating the process locale.
- `cargo build -p roastty` — no warnings.
- `cargo test -p roastty` — full Roastty test suite stays green.
- `cargo fmt -p roastty -- --check` — clean.
- no-ghostty grep on touched source — clean.
- `git diff --check` — clean.

Pass = Roastty has the upstream `ensureLocale` sequence in Rust with
deterministic tests and a thin real `setlocale` boundary.

## Design Review

**Reviewer:** Codex (gpt-5.5, medium) · resumed session
`019e8f83-9029-7d43-8e82-f4c5754e14ba`

**Verdict:** APPROVED.

Initial review found two Required issues. First, the design probed `LANGUAGE`
whenever initial `LANG` was missing or empty, even if the Cocoa `LANG` probe
failed; upstream returns early from `setLangFromCocoa` when language or country
is missing, so preferred languages are only consulted after `LANG` is available.
Second, the proposed `real_setlocale(Some("C"))` smoke test would mutate the
process-global C locale.

The corrected design nests `LANGUAGE` probing inside the successful Cocoa `LANG`
branch and adds a fake-driven test that `LANGUAGE` is not probed or set when the
`LANG` probe returns `None`. The real smoke test is now query-only:
`real_setlocale(None)` passes a null pointer and does not mutate locale state.
Follow-up review approved the fake boundary and branch coverage with no
remaining Required changes.

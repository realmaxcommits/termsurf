//! POSIX PTY ownership and sizing.

use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PtySize {
    pub(crate) rows: u16,
    pub(crate) cols: u16,
    pub(crate) width_px: u16,
    pub(crate) height_px: u16,
}

impl PtySize {
    fn winsize(self) -> libc::winsize {
        libc::winsize {
            ws_row: self.rows,
            ws_col: self.cols,
            ws_xpixel: self.width_px,
            ws_ypixel: self.height_px,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Pty {
    master: OwnedFd,
    slave: OwnedFd,
}

impl Pty {
    pub(crate) fn open(size: PtySize) -> io::Result<Self> {
        let mut master = 0;
        let mut slave = 0;
        let mut winsize = size.winsize();
        if unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut winsize,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }

        // Take ownership immediately so any post-open error closes both descriptors.
        let master = unsafe { OwnedFd::from_raw_fd(master) };
        let slave = unsafe { OwnedFd::from_raw_fd(slave) };

        set_cloexec(master.as_raw_fd())?;
        set_cloexec(slave.as_raw_fd())?;

        Ok(Self { master, slave })
    }

    pub(crate) fn set_size(&self, size: PtySize) -> io::Result<()> {
        let winsize = size.winsize();
        if unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &winsize) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub(crate) fn master_fd(&self) -> RawFd {
        self.master.as_raw_fd()
    }

    pub(crate) fn slave_fd(&self) -> RawFd {
        self.slave.as_raw_fd()
    }
}

fn set_cloexec(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_size() -> PtySize {
        PtySize {
            rows: 24,
            cols: 80,
            width_px: 800,
            height_px: 600,
        }
    }

    fn pty_size(fd: RawFd) -> io::Result<PtySize> {
        let mut winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut winsize) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(PtySize {
            rows: winsize.ws_row,
            cols: winsize.ws_col,
            width_px: winsize.ws_xpixel,
            height_px: winsize.ws_ypixel,
        })
    }

    fn fd_cloexec(fd: RawFd) -> bool {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        assert!(flags >= 0, "F_GETFD failed");
        flags & libc::FD_CLOEXEC != 0
    }

    struct RawModeGuard {
        fd: RawFd,
        original: libc::termios,
    }

    impl RawModeGuard {
        fn new(fd: RawFd) -> io::Result<Self> {
            let mut original = unsafe { std::mem::zeroed::<libc::termios>() };
            if unsafe { libc::tcgetattr(fd, &mut original) } < 0 {
                return Err(io::Error::last_os_error());
            }
            let mut raw = original;
            unsafe {
                libc::cfmakeraw(&mut raw);
            }
            if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { fd, original })
        }
    }

    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            unsafe {
                libc::tcsetattr(self.fd, libc::TCSANOW, &self.original);
            }
        }
    }

    #[test]
    fn pty_open_returns_valid_descriptors() {
        let pty = Pty::open(test_size()).expect("open pty");

        assert!(pty.master_fd() >= 0);
        assert!(pty.slave_fd() >= 0);
        assert_ne!(pty.master_fd(), pty.slave_fd());
    }

    #[test]
    fn pty_open_sets_cloexec_on_both_descriptors() {
        let pty = Pty::open(test_size()).expect("open pty");

        assert!(fd_cloexec(pty.master_fd()));
        assert!(fd_cloexec(pty.slave_fd()));
    }

    #[test]
    fn pty_open_applies_initial_size() {
        let pty = Pty::open(test_size()).expect("open pty");

        assert_eq!(
            pty_size(pty.master_fd()).expect("get pty size"),
            test_size()
        );
    }

    #[test]
    fn pty_set_size_updates_reported_size() {
        let pty = Pty::open(test_size()).expect("open pty");
        let resized = PtySize {
            rows: 40,
            cols: 120,
            width_px: 1200,
            height_px: 900,
        };

        pty.set_size(resized).expect("set pty size");

        assert_eq!(pty_size(pty.master_fd()).expect("get pty size"), resized);
    }

    #[test]
    fn pty_transfers_bytes_without_blocking() {
        let pty = Pty::open(test_size()).expect("open pty");
        let _raw_mode = RawModeGuard::new(pty.slave_fd()).expect("raw mode");
        let msg = b"hi";

        let written = unsafe {
            libc::write(
                pty.slave_fd(),
                msg.as_ptr() as *const libc::c_void,
                msg.len(),
            )
        };
        assert_eq!(written, msg.len() as isize);

        let mut pollfd = libc::pollfd {
            fd: pty.master_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pollfd, 1, 100) };
        assert_eq!(ready, 1, "pty master did not become readable");
        assert_ne!(pollfd.revents & libc::POLLIN, 0);

        let mut buf = [0u8; 2];
        let got = unsafe {
            libc::read(
                pty.master_fd(),
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
            )
        };
        assert_eq!(got, 2);
        assert_eq!(&buf, msg);
    }
}

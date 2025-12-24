// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2017-2018 Joshua Liebow-Feeser <hello@joshlf.com>
// SPDX-FileCopyrightText: 2025 cainthebest <https://github.com/cainthebest>

use {
    core::hint::{likely, unlikely},
    libc::{EINTR, c_int, size_t, ssize_t},
};

pub(crate) struct UnixBackend;

impl UnixBackend {
    #[inline(always)]
    fn errno() -> i32 {
        #[cfg(target_os = "macos")]
        unsafe {
            *libc::__error()
        }

        #[cfg(not(target_os = "macos"))]
        unsafe {
            *libc::__errno_location()
        }
    }

    #[inline(always)]
    fn write_fd(fd: c_int, bytes: &[u8]) {
        if unlikely(bytes.is_empty()) {
            return;
        }

        let mut off: usize = 0;
        let len = bytes.len();

        unsafe {
            while off < len {
                let ptr = bytes.as_ptr().add(off) as *const _;
                let n = (len - off) as size_t;

                let r: ssize_t = libc::write(fd, ptr, n);

                if likely(r > 0) {
                    off += r as usize;

                    continue;
                }

                if r == -1 && Self::errno() == EINTR {
                    continue;
                }

                Self::abort();
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn abort() -> ! {
        unsafe { libc::abort() }
    }
}

impl super::Backend for UnixBackend {
    #[inline(always)]
    fn write_out(s: &str) {
        Self::write_fd(1, s.as_bytes());
    }

    #[inline(always)]
    fn write_err(s: &str) {
        Self::write_fd(2, s.as_bytes());
    }

    #[cold]
    #[inline(always)]
    fn abort() -> ! {
        Self::abort()
    }
}

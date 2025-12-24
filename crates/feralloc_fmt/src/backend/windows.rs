// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2025 cainthebest <https://github.com/cainthebest>

use {
    core::{
        hint::{likely, unlikely},
        intrinsics::abort as hard_abort,
        ptr,
    },
    windows_sys::Win32::{
        Foundation::{HANDLE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::WriteFile,
        System::{
            Console::{GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
            Threading::{GetCurrentProcess, TerminateProcess},
        },
    },
};

pub(crate) struct WindowsBackend;

impl WindowsBackend {
    const MAX_CHUNK: usize = u32::MAX as usize;

    #[inline(always)]
    fn write(h: HANDLE, bytes: &[u8]) {
        if unlikely(bytes.is_empty()) {
            return;
        }

        if unlikely(h.is_null() || h == INVALID_HANDLE_VALUE) {
            return;
        }

        unsafe {
            let len = bytes.len();

            if likely(len <= Self::MAX_CHUNK) {
                let mut written: u32 = 0;
                let ok = WriteFile(
                    h,
                    bytes.as_ptr() as *const _,
                    len as u32,
                    &mut written,
                    ptr::null_mut(),
                );

                let fail = ok == 0;
                let partial = (written as usize) != len;

                if unlikely(fail | partial) {
                    if unlikely(fail || written == 0) {
                        Self::abort();
                    }

                    let mut off = written as usize;
                    while off < len {
                        let rem = len - off;

                        let mut w: u32 = 0;
                        let ok = WriteFile(
                            h,
                            bytes.as_ptr().add(off) as *const _,
                            rem as u32,
                            &mut w,
                            ptr::null_mut(),
                        );

                        if unlikely(ok == 0 || w == 0) {
                            Self::abort();
                        }

                        off += w as usize;
                    }
                }

                return;
            }

            let mut off = 0usize;
            while off < len {
                let rem = len - off;
                let chunk = if rem > Self::MAX_CHUNK {
                    Self::MAX_CHUNK
                } else {
                    rem
                };

                let mut w: u32 = 0;
                let ok = WriteFile(
                    h,
                    bytes.as_ptr().add(off) as *const _,
                    chunk as u32,
                    &mut w,
                    ptr::null_mut(),
                );

                if unlikely(ok == 0 || w == 0) {
                    Self::abort();
                }

                off += w as usize;
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn abort() -> ! {
        unsafe {
            let _ = TerminateProcess(GetCurrentProcess(), 134);
        }

        hard_abort()
    }
}

impl super::Backend for WindowsBackend {
    #[inline(always)]
    fn write_out(s: &str) {
        Self::write(unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }, s.as_bytes());
    }

    #[inline(always)]
    fn write_err(s: &str) {
        Self::write(unsafe { GetStdHandle(STD_ERROR_HANDLE) }, s.as_bytes());
    }

    #[cold]
    #[inline(always)]
    fn abort() -> ! {
        Self::abort()
    }
}

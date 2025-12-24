// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2025 cainthebest <https://github.com/cainthebest>

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) type CurrentBackend = windows::WindowsBackend;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(crate) type CurrentBackend = unix::UnixBackend;

pub(crate) trait Backend {
    fn write_out(s: &str);

    fn write_err(s: &str);

    fn abort() -> !;
}

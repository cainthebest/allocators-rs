// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2025 cainthebest <https://github.com/cainthebest>

//! Backend layer for `feralloc_fmt`.
//!
//! This module defines the *contract* between the formatting engine and the
//! platform specific output implementation.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) type CurrentBackend = windows::WindowsBackend;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(crate) type CurrentBackend = unix::UnixBackend;



/// A backend is the lowest-level output/abort provider.
///
/// **Hard requirements:**
/// - Must not allocate (directly or indirectly)
/// - Must not panic / unwind
/// - Must be safe to call from inside the allocator, OOM paths, and panic paths
///
/// Backends are ZSTs and selected at compile time.
pub(crate) trait Backend {
    /// Write a UTF-8 chunk to the primary output (stdout-like).
    ///
    /// Called frequently and potentially with small chunks (fmt streams).
    fn write_out(s: &str);

    /// Write a UTF-8 chunk to the error output (stderr-like).
    ///
    /// If the platform has no separate stderr, forward to `write_out`.
    fn write_err(s: &str);

    /// Abort immediately. Must never return.
    fn abort() -> !;
}

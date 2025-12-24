// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2017-2018 Joshua Liebow-Feeser <hello@joshlf.com>
// SPDX-FileCopyrightText: 2025 cainthebest <https://github.com/cainthebest>

#![no_std]
#![allow(internal_features)]
#![feature(likely_unlikely, core_intrinsics)]

mod backend;
mod runtime;

#[cfg(not(any(windows, unix)))]
compile_error!("feralloc_fmt only supports Windows and Unix targets.");

#[cfg(not(target_has_atomic = "8"))]
compile_error!("feralloc_fmt requires 8-bit atomics.");

#[doc(hidden)]
pub use runtime::{
    fatal_panic as __fatal_panic, write_err_fmt as __write_err_fmt,
    write_out_fmt as __write_out_fmt,
};

#[doc(hidden)]
pub fn __unlikely(b: bool) -> bool {
    core::hint::unlikely(b)
}

#[macro_export]
macro_rules! feralloc_print {
    ($($arg:tt)*) => {{
        $crate::__write_out_fmt(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! feralloc_eprint {
    ($($arg:tt)*) => {{
        $crate::__write_err_fmt(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! feralloc_println {
    () => { $crate::feralloc_print!("\n") };
    ($fmt:expr) => { $crate::feralloc_print!(concat!($fmt, "\n")) };
    ($fmt:expr, $($arg:tt)*) => { $crate::feralloc_print!(concat!($fmt, "\n"), $($arg)*) };
}

#[macro_export]
macro_rules! feralloc_eprintln {
    () => { $crate::feralloc_eprint!("\n") };
    ($fmt:expr) => { $crate::feralloc_eprint!(concat!($fmt, "\n")) };
    ($fmt:expr, $($arg:tt)*) => { $crate::feralloc_eprint!(concat!($fmt, "\n"), $($arg)*) };
}

#[macro_export]
macro_rules! feralloc_panic {
    () => {{
        $crate::__fatal_panic(core::format_args!("explicit panic"), file!(), line!(), column!())
    }};
    ($msg:expr) => {{
        $crate::__fatal_panic(core::format_args!($msg), file!(), line!(), column!())
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $crate::__fatal_panic(core::format_args!($fmt, $($arg)*), file!(), line!(), column!())
    }};
}

#[macro_export]
macro_rules! feralloc_assert {
    ($pred:expr $(,)?) => {{
        if $crate::__unlikely(!($pred))  {
            $crate::feralloc_panic!("assertion failed: {}", stringify!($pred));
        }
    }};
    ($pred:expr, $($arg:tt)+) => {{
        if $crate::__unlikely(!($pred))  {
            $crate::feralloc_panic!($($arg)+);
        }
    }};
}

#[macro_export]
macro_rules! feralloc_debug_assert {
    ($($arg:tt)*) => {{
        if cfg!(debug_assertions) {
            $crate::feralloc_assert!($($arg)*);
        }
    }};
}

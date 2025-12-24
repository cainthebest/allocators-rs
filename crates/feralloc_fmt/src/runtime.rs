// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2017-2018 Joshua Liebow-Feeser <hello@joshlf.com>
// SPDX-FileCopyrightText: 2025 cainthebest <https://github.com/cainthebest>

use {
    crate::backend::{Backend, CurrentBackend},
    core::{
        fmt::{self, Arguments},
        hint::{likely, spin_loop, unlikely},
        marker::PhantomData,
        sync::atomic::{AtomicU8, Ordering},
    },
};

const LOCKED: u8 = 0b0000_0001;
const PANICKING: u8 = 0b0000_0010;

static STATE: AtomicU8 = AtomicU8::new(0);

struct Guard {
    locked: bool,
}

impl Drop for Guard {
    #[inline(always)]
    fn drop(&mut self) {
        if self.locked {
            STATE.fetch_and(!LOCKED, Ordering::Release);
        }
    }
}

struct Lock;

impl Lock {
    #[inline(always)]
    fn acquire() -> Guard {
        let mut st = STATE.load(Ordering::Relaxed);

        if unlikely((st & PANICKING) != 0) {
            return Guard { locked: false };
        }

        if (st & LOCKED) == 0
            && likely(
                STATE
                    .compare_exchange(st, st | LOCKED, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok(),
            )
        {
            return Guard { locked: true };
        }

        loop {
            loop {
                st = STATE.load(Ordering::Relaxed);
                if unlikely((st & PANICKING) != 0) {
                    return Guard { locked: false };
                }

                if (st & LOCKED) == 0 {
                    break;
                }

                spin_loop();
            }

            if STATE
                .compare_exchange_weak(st, st | LOCKED, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Guard { locked: true };
            }
        }
    }
}

#[cold]
#[inline(never)]
fn enter_panic_or_abort() {
    let prev = STATE.fetch_or(PANICKING, Ordering::Relaxed);
    if (prev & PANICKING) != 0 {
        CurrentBackend::write_err("panic while panicking\n");
        CurrentBackend::abort();
    }

    STATE.fetch_and(!LOCKED, Ordering::Relaxed);
}

struct Writer {
    is_err: bool,
    _z: PhantomData<fn() -> CurrentBackend>,
}

impl Writer {
    const OUT: Self = Self {
        is_err: false,
        _z: PhantomData,
    };

    const ERR: Self = Self {
        is_err: true,
        _z: PhantomData,
    };
}

impl fmt::Write for Writer {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.is_err {
            CurrentBackend::write_err(s);
        } else {
            CurrentBackend::write_out(s);
        }

        Ok(())
    }
}

#[inline(always)]
pub fn write_out_fmt(args: Arguments) {
    let _g = Lock::acquire();
    let mut w = Writer::OUT;
    let _ = fmt::write(&mut w, args);
}

#[inline(always)]
pub fn write_err_fmt(args: Arguments) {
    let _g = Lock::acquire();
    let mut w = Writer::ERR;
    let _ = fmt::write(&mut w, args);
}

#[inline(always)]
fn write_u32_err(mut x: u32) {
    let mut buf = [0u8; 10];
    let mut i = 10;
    loop {
        let d = (x % 10) as u8;
        x /= 10;
        i -= 1;
        buf[i] = b'0' + d;
        if x == 0 {
            break;
        }
    }
    let s = unsafe { core::str::from_utf8_unchecked(&buf[i..]) };
    CurrentBackend::write_err(s);
}

#[cold]
#[inline(never)]
pub fn fatal_panic(args: Arguments, file: &'static str, line: u32, col: u32) -> ! {
    enter_panic_or_abort();

    CurrentBackend::write_err("thread panicked at '");
    {
        let _g = Lock::acquire();
        let mut w = Writer::ERR;
        let _ = fmt::write(&mut w, args);
    }
    CurrentBackend::write_err("', ");
    CurrentBackend::write_err(file);
    CurrentBackend::write_err(":");
    write_u32_err(line);
    CurrentBackend::write_err(":");
    write_u32_err(col);
    CurrentBackend::write_err("\n");

    CurrentBackend::abort()
}

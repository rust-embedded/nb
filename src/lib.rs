#![doc = include_str!("../README.md")]
//!
//! # How to use this crate
//!
//! Application specific errors can be put inside the `Other` variant in the
//! [`nb::Error`](enum.Error.html) enum.
//!
//! So in your API instead of returning `Result<T, MyError>` return
//! `nb::Result<T, MyError>`
//!
//! ```
//! enum MyError {
//!     ThisError,
//!     ThatError,
//!     // ..
//! }
//!
//! // This is a blocking function, so it returns a normal `Result`
//! fn before() -> Result<(), MyError> {
//!     // ..
//! #   Ok(())
//! }
//!
//! // This is now a potentially (read: *non*) blocking function so it returns `nb::Result`
//! // instead of blocking
//! fn after() -> nb::Result<(), MyError> {
//!     // ..
//! #   Ok(())
//! }
//! ```
//!
//! You can use `Infallible` to signal that some API has no fatal
//! errors but may block:
//!
//! ```
//! use core::convert::Infallible;
//!
//! // This returns `Ok(())` or `Err(nb::Error::WouldBlock)`
//! fn maybe_blocking_api() -> nb::Result<(), Infallible> {
//!     // ..
//! #   Ok(())
//! }
//! ```
//!
//! Once your API uses [`nb::Result`] you can leverage the [`block!`], macro
//! to adapt it for blocking operation, or handle scheduling yourself. You can
//! also use the [`fut!`] macro to use it in an async/await context
//!
//! [`block!`]: macro.block.html
//! [`fut!`]: macro.fut.html
//! [`nb::Result`]: type.Result.html
//!
//! # Examples
//!
//! ## A Core I/O API
//!
//! Imagine the code (crate) below represents a Hardware Abstraction Layer for some microcontroller
//! (or microcontroller family).
//!
//! *In this and the following examples let's assume for simplicity that peripherals are treated
//! as global singletons and that no preemption is possible (i.e. interrupts are disabled).*
//!
//! ```
//! # use core::convert::Infallible;
//! // This is the `hal` crate
//! use nb;
//!
//! /// An LED
//! pub struct Led;
//!
//! impl Led {
//!     pub fn off(&self) {
//!         // ..
//!     }
//!     pub fn on(&self) {
//!         // ..
//!     }
//! }
//!
//! /// Serial interface
//! pub struct Serial;
//! pub enum Error {
//!     Overrun,
//!     // ..
//! }
//!
//! impl Serial {
//!     /// Reads a single byte from the serial interface
//!     pub fn read(&self) -> nb::Result<u8, Error> {
//!         // ..
//! #       Ok(0)
//!     }
//!
//!     /// Writes a single byte to the serial interface
//!     pub fn write(&self, byte: u8) -> nb::Result<(), Error> {
//!         // ..
//! #       Ok(())
//!     }
//! }
//!
//! /// A timer used for timeouts
//! pub struct Timer;
//!
//! impl Timer {
//!     /// Waits until the timer times out
//!     pub fn wait(&self) -> nb::Result<(), Infallible> {
//!         //^ NOTE the `Infallible` indicates that this operation can block but has no
//!         //  other form of error
//!
//!         // ..
//! #       Ok(())
//!     }
//! }
//! ```
//!
//! ## Blocking mode
//!
//! Turn on an LED for one second and *then* loops back serial data.
//!
//! ```
//! use core::convert::Infallible;
//! use nb::block;
//!
//! use hal::{Led, Serial, Timer};
//!
//! # fn main() -> Result<(), Infallible> {
//! // Turn the LED on for one second
//! Led.on();
//! block!(Timer.wait())?;
//! Led.off();
//!
//! // Serial interface loopback
//! # return Ok(());
//! loop {
//!     let byte = block!(Serial.read())?;
//!     block!(Serial.write(byte))?;
//! }
//! # }
//!
//! # mod hal {
//! #   use nb;
//! #   use core::convert::Infallible;
//! #   pub struct Led;
//! #   impl Led {
//! #       pub fn off(&self) {}
//! #       pub fn on(&self) {}
//! #   }
//! #   pub struct Serial;
//! #   impl Serial {
//! #       pub fn read(&self) -> nb::Result<u8, Infallible> { Ok(0) }
//! #       pub fn write(&self, _: u8) -> nb::Result<(), Infallible> { Ok(()) }
//! #   }
//! #   pub struct Timer;
//! #   impl Timer {
//! #       pub fn wait(&self) -> nb::Result<(), Infallible> { Ok(()) }
//! #   }
//! # }
//! ```
//! ## Future mode
//!
//! Turn on an LED for one second and *then* loops back serial data.
//!
//! ```
//! use core::convert::Infallible;
//! use nb::fut;
//!
//! use hal::{Led, Serial, Timer};
//!
//! # async fn run() -> Result<(), Infallible>{
//! Led.on();
//! fut!(Timer.wait()).await;
//! Led.off();
//! loop {
//!     let byte = fut!(Serial.read()).await?;
//!     fut!(Serial.write(byte)).await?;
//! }
//! # }
//!
//! # mod hal {
//! #   use nb;
//! #   use core::convert::Infallible;
//! #   pub struct Led;
//! #   impl Led {
//! #       pub fn off(&self) {}
//! #       pub fn on(&self) {}
//! #   }
//! #   pub struct Serial;
//! #   impl Serial {
//! #       pub fn read(&self) -> nb::Result<u8, Infallible> { Ok(0) }
//! #       pub fn write(&self, _: u8) -> nb::Result<(), Infallible> { Ok(()) }
//! #   }
//! #   pub struct Timer;
//! #   impl Timer {
//! #       pub fn wait(&self) -> nb::Result<(), Infallible> { Ok(()) }
//! #   }
//! # }
//!
//! # Features
//!
//! - `defmt-0-3` - unstable feature which adds [`defmt::Format`] impl for [`Error`].

#![no_std]

use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// A non-blocking result
pub type Result<T, E> = ::core::result::Result<T, Error<E>>;

/// A non-blocking error
///
/// The main use of this enum is to add a `WouldBlock` variant to an existing
/// error enum.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error<E> {
    /// A different kind of error
    Other(E),
    /// This operation requires blocking behavior to complete
    WouldBlock,
}

#[cfg(feature = "defmt-0-3")]
impl<E> defmt::Format for Error<E>
where
    E: defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        match *self {
            Error::Other(ref e) => defmt::Format::format(e, f),
            Error::WouldBlock => defmt::write!(f, "WouldBlock",),
        }
    }
}

impl<E> fmt::Debug for Error<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Other(ref e) => fmt::Debug::fmt(e, f),
            Error::WouldBlock => f.write_str("WouldBlock"),
        }
    }
}

impl<E> Error<E> {
    /// Maps an `Error<E>` to `Error<T>` by applying a function to a contained
    /// `Error::Other` value, leaving an `Error::WouldBlock` value untouched.
    pub fn map<T, F>(self, op: F) -> Error<T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Error::Other(e) => Error::Other(op(e)),
            Error::WouldBlock => Error::WouldBlock,
        }
    }
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Error<E> {
        Error::Other(error)
    }
}

/// Turns the non-blocking expression `$e` into a blocking operation.
///
/// This is accomplished by continuously calling the expression `$e` until it no
/// longer returns `Error::WouldBlock`
///
/// # Input
///
/// An expression `$e` that evaluates to `nb::Result<T, E>`
///
/// # Output
///
/// - `Ok(t)` if `$e` evaluates to `Ok(t)`
/// - `Err(e)` if `$e` evaluates to `Err(nb::Error::Other(e))`
#[macro_export]
macro_rules! block {
    ($e:expr) => {
        loop {
            #[allow(unreachable_patterns)]
            match $e {
                Err($crate::Error::Other(e)) =>
                {
                    #[allow(unreachable_code)]
                    break Err(e)
                }
                Err($crate::Error::WouldBlock) => {}
                Ok(x) => break Ok(x),
            }
        }
    };
}

pub struct NbFuture<Ok, Err, Gen: FnMut() -> Result<Ok, Err>> {
    gen: Gen,
}

impl<Ok, Err, Gen: FnMut() -> Result<Ok, Err>> From<Gen> for NbFuture<Ok, Err, Gen> {
    fn from(gen: Gen) -> Self {
        Self { gen }
    }
}

impl<Ok, Err, Gen: FnMut() -> Result<Ok, Err>> Future for NbFuture<Ok, Err, Gen> {
    type Output = core::result::Result<Ok, Err>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let gen = unsafe { &mut self.get_unchecked_mut().gen };
        let res = gen();

        match res {
            Ok(res) => Poll::Ready(Ok(res)),
            Err(Error::WouldBlock) => Poll::Pending,
            Err(Error::Other(err)) => Poll::Ready(Err(err)),
        }
    }
}

/// The equivalent of [block], expect instead of blocking this creates a
/// [Future] which can `.await`ed.
///
/// # Input
///
/// An expression `$e` that evaluates to `nb::Result<T, E>`
///
/// # Output
///
/// - `Ok(t)` if `$e` evaluates to `Ok(t)`
/// - `Err(e)` if `$e` evaluates to `Err(nb::Error::Other(e))`
#[macro_export]
macro_rules! fut {
    ($call:expr) => {{
        nb::NbFuture::from(|| $call)
    }};
}

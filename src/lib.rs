//! Minimal and reusable non-blocking I/O layer
//!
//! The ultimate goal of this crate is *code reuse*. With this crate you can
//! write *core* I/O APIs that can then be adapted to operate in either blocking
//! or non-blocking manner. Furthermore those APIs are not tied to a particular
//! asynchronous model and can be adapted to work with the `futures` model or
//! with the `async` / `await` model.
//!
//! # Core idea
//!
//! The [`WouldBlock`](enum.Error.html) error variant signals that the operation can't be completed
//! *right now* and would need to block to complete. [`WouldBlock`](enum.Error.html) is a special
//! error in the sense that's not *fatal*; the operation can still be completed by retrying again
//! later.
//! 
//! [`nb::Result`](type.Result.html) is based on the API of
//! [`std::io::Result`](https://doc.rust-lang.org/std/io/type.Result.html), which has a
//! `WouldBlock` variant in its
//! [`ErrorKind`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html). 
//!
//! We can map [`WouldBlock`](enum.Error.html) to different blocking and non-blocking models:
//! - In blocking mode: [`WouldBlock`](enum.Error.html) means try again right now (i.e. busy wait)
//! - In `futures` mode: [`WouldBlock`](enum.Error.html) means
//!   [`Async::NotReady`](https://docs.rs/futures)
//! - In `await` mode: [`WouldBlock`](enum.Error.html) means `yield` (suspend the generator)
//!
//! # How to use this crate
//! Application specific errors can be put inside the `Other` variant in the
//! [`nb::Error`](enum.Error.html) enum.
//!
//! So in your API instead of returning `Result<T, MyError>` return
//! `nb::Result<T, MyError>` 
//! 
//! ``` ignore
//! enum MyError { ThisError, ThatError, .. }
//!
//! // This is NOT a blocking function, so it returns a normal `Result`
//! fn before() -> Result<(), MyError> { .. }
//!
//! // This is a potentially blocking function so it returns `nb::Result`
//! fn after() -> nb::Result<(), MyError> { .. }
//! ```
//!
//! You can use the *never type* (`!`) to signal that some API has no fatal
//! errors but may block:
//!
//! ``` ignore
//! // This returns `Ok(())` or `Err(nb::Error::WouldBlock)`
//! fn maybe_blocking_api() -> nb::Result<(), !> { .. }
//! ```
//!
//! Once your API uses [`nb::Result`](type.Result.html) you can leverage the [`block!`],
//! [`try_nb!`] and [`await!`] macros to adapt it for blocking operation, or for
//! non-blocking operation with `futures` or `await`.
//!
//! [`block!`]: macro.block.html
//! [`try_nb!`]: macro.try_nb.html
//! [`await!`]: macro.await.html
//!
//! # Examples
//!
//! ## Core I/O API
//!
//! A Hardware Abstraction Layer for some microcontroller (or microcontroller
//! family).
//!
//! In this and the following examples let's assume for simplicity that
//! peripherals are represented by global singletons and that no preemption is
//! possible (i.e. no interrupts).
//!
//! ``` ignore
//! // This is the `hal` crate
//! // Note that it doesn't depend on the `futures` crate
//!
//! extern crate nb;
//!
//! /// An LED
//! pub struct Led;
//!
//! impl Led {
//!     pub fn off(&self) { .. }
//!     pub fn on(&self) { .. }
//! }
//!
//! /// Serial interface
//! pub struct Serial;
//! pub enum Error { Overrun, .. }
//!
//! impl Serial {
//!     /// Reads a single byte from the serial interface
//!     pub fn read(&self) -> nb::Result<u8, Error> { .. }
//!
//!     /// Writes a single byte to the serial interface
//!     pub fn write(&self, byte: u8) -> nb::Result<(), Error> { .. }
//! }
//!
//! /// A timer used for timeouts
//! pub struct Timer;
//!
//! impl Timer {
//!     /// Waits until the timer times out
//!     pub fn wait(&self) -> nb::Result<(), !> { .. }
//!     //^ NOTE the `!` indicates that this operation can block but has no
//!     //  other form of error
//! }
//! ```
//!
//! ## Blocking mode
//!
//! Turn on an LED for one second and *then* loops back serial data.
//!
//! ``` ignore
//! extern crate hal;
//! #[macro_use]
//! extern crate nb;
//!
//! use hal::{Led, Serial, Timer};
//!
//! // Turn the LED on for one second
//! Led.on();
//! block!(Timer.wait()).unwrap(); // NOTE(unwrap) E = !
//! Led.off();
//!
//! // Serial interface loopback
//! loop {
//!     let byte = block!(Serial.read());
//!     block!(Serial.write(byte));
//! }
//! ```
//!
//! ## `futures`
//!
//! Blinks an LED every second *and* loops back serial data. Both tasks run
//! concurrently.
//!
//! ``` ignore
//! extern crate futures;
//! extern crate hal;
//! #[macro_use]
//! extern crate nb;
//!
//! use futures::{Async, Future, future};
//! use hal::{Led, Serial, Timer};
//!
//! /// `futures` version of `Timer.wait`
//! ///
//! /// This returns a future that must be polled to completion
//! fn wait() -> impl Future<Item = (), Error = !> {
//!     future::poll_fn(|| {
//!         Ok(Async::Ready(try_nb!(Timer.wait())))
//!     })
//! }
//!
//! /// `futures` version of `Serial.read`
//! ///
//! /// This returns a future that must be polled to completion
//! fn read() -> impl Future<Item = u8, Error = Error> {
//!     future::poll_fn(|| {
//!         Ok(Async::Ready(try_nb!(Serial.read())))
//!     })
//! }
//!
//! /// `futures` version of `Serial.write`
//! ///
//! /// This returns a future that must be polled to completion
//! fn write(byte: u8) -> impl Future<Item = (), Error = Error> {
//!     future::poll_fn(|| {
//!         Ok(Async::Ready(try_nb!(Serial.write(byte))))
//!     })
//! }
//!
//! // Tasks
//! let mut blinky = future::loop_fn(true, |_| {
//!     wait().map(|_| {
//!         if state {
//!             Led.on();
//!         } else {
//!             Led.off();
//!         }
//!
//!         Loop::Continue(!state)
//!     });
//! });
//!
//! let mut loopback = future::loop_fn((), |_| {
//!     read().and_then(|byte| {
//!         write(byte)
//!     }).map(|_| {
//!         Loop::Continue(())
//!     });
//! });
//!
//! // Event loop
//! loop {
//!     blinky().poll().unwrap(); // NOTE(unwrap) E = !
//!     loopback().poll().unwrap();
//! }
//! ```
//!
//! ## `await!`
//!
//! **NOTE** The `await!` macro requires language support for generators, which
//! is not yet in the compiler.
//!
//! This is equivalent to the `futures` example but with much less boilerplate.
//!
//! ``` ignore
//! extern crate hal;
//! #[macro_use]
//! extern crate nb;
//!
//! use hal::{Led, Serial, Timer};
//!
//! // Tasks
//! let mut blinky = (|| {
//!     let mut state = false;
//!     loop {
//!         // `await!` means suspend / yield instead of blocking
//!         await!(Timer.wait()).unwrap(); // NOTE(unwrap) E = !
//!
//!         state = !state;
//!
//!         if state {
//!              Led.on();
//!         } else {
//!              Led.off();
//!         }
//!     }
//! })();
//!
//! let mut loopback = (|| {
//!     loop {
//!         let byte = await!(serial.read()).unwrap();
//!         await!(serial.write(byte)).unwrap();
//!     }
//! })();
//!
//! // Event loop
//! loop {
//!     blinky.resume();
//!     serial.resume();
//! }
//! ```

#![no_std]
#![deny(warnings)]

use core::fmt;

/// A non-blocking result
pub type Result<T, E> = ::core::result::Result<T, Error<E>>;

/// A non-blocking error
///
/// The main use of this enum is to add a `WouldBlock` variant to an existing
/// error enum.
pub enum Error<E> {
    /// A different kind of error
    Other(E),
    /// This operation requires blocking behavior to complete
    WouldBlock,
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

/// Await operation (*won't work until the language gains support for
/// generators*)
///
/// This macro evaluates the expression `$e` *cooperatively* yielding control
/// back to the (generator) caller whenever `$e` evaluates to
/// `Error::WouldBlock`.
///
/// # Requirements
///
/// This macro must be called within a generator body.
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
macro_rules! await {
    ($e:expr) => {
        loop {
            match $e {
                Err($crate::Error::Other(e)) => break Err(e),
                Err($crate::Error::WouldBlock) => yield (),
                Ok(x) => break Ok(x),
            }
        }
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
            match $e {
                Err($crate::Error::Other(e)) => break Err(e),
                Err($crate::Error::WouldBlock) => {},
                Ok(x) => break Ok(x),
            }
        }
    }
}

/// Future adapter
///
/// This is a *try* operation from a `nb::Result` to a `futures::Poll`
///
/// # Requirements
///
/// This macro must be called within a function / closure that has signature
/// `fn(..) -> futures::Poll<T, E>`.
///
/// This macro requires that the [`futures`] crate is in the root of the crate.
///
/// [`futures`]: https://crates.io/crates/futures
///
/// # Input
///
/// An expression `$e` that evaluates to `nb::Result<T, E>`
///
/// # Early return
///
/// - `Ok(Async::NotReady)` if `$e` evaluates to `Err(nb::Error::WouldBlock)`
/// - `Err(e)` if `$e` evaluates to `Err(nb::Error::Other(e))`
///
/// # Output
///
/// `t` if `$e` evaluates to `Ok(t)`
#[macro_export]
macro_rules! try_nb {
    ($e:expr) => {
        match $e {
            Err($crate::Error::Other(e)) => return Err(e),
            Err($crate::Error::WouldBlock) => {
                return Ok(::futures::Async::NotReady)
            },
            Ok(x) => x,
        }
    }
}

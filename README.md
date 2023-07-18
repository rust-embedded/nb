[![crates.io](https://img.shields.io/crates/d/nb.svg)](https://crates.io/crates/nb)
[![crates.io](https://img.shields.io/crates/v/nb.svg)](https://crates.io/crates/nb)
[![Documentation](https://docs.rs/nb/badge.svg)](https://docs.rs/nb)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.62+-blue.svg)

# `nb`

> Minimal and reusable non-blocking I/O layer

This project is developed and maintained by the [HAL team][team].

## [Documentation](https://docs.rs/nb)

The ultimate goal of this crate is *code reuse*. With this crate you can
write *core* I/O APIs that can then be adapted to operate in either blocking
or non-blocking manner. Furthermore those APIs are not tied to a particular
asynchronous model and can be adapted to work with the `futures` model or
with the `async` / `await` model.

### Core idea

The [`WouldBlock`](enum.Error.html) error variant signals that the operation
can't be completed *right now* and would need to block to complete.
[`WouldBlock`](enum.Error.html) is a special error in the sense that's not
*fatal*; the operation can still be completed by retrying again later.

[`nb::Result`](type.Result.html) is based on the API of
[`std::io::Result`](https://doc.rust-lang.org/std/io/type.Result.html),
which has a `WouldBlock` variant in its
[`ErrorKind`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html).

We can map [`WouldBlock`](enum.Error.html) to different blocking and
non-blocking models:

- In blocking mode: [`WouldBlock`](enum.Error.html) means try again right
  now (i.e. busy wait)
- In `futures` mode: [`WouldBlock`](enum.Error.html) means
  [`Async::NotReady`](https://docs.rs/futures)
- In `await` mode: [`WouldBlock`](enum.Error.html) means `yield`
  (suspend the generator)


## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.62 and up. It *might*
compile with older versions but that may change in any new patch release.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], the maintainer of this crate, the [HAL team][team], promises
to intervene to uphold that code of conduct.

[CoC]: CODE_OF_CONDUCT.md
[team]: https://github.com/rust-embedded/wg#the-hal-team

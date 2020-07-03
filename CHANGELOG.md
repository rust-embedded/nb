# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v1.0.0]

### Changed

- [breaking-change] The `unstable` feature and its code has been removed.
  This includes the macros `try_nb!` and `await!`.

## [v0.1.1] - 2018-01-10

### Fixed

- The `await!` macro now works when the expression `$e` mutably borrows `self`.

## v0.1.0 - 2018-01-10

Initial release

[Unreleased]: https://github.com/japaric/nb/compare/v0.1.1...HEAD
[v0.1.1]: https://github.com/japaric/nb/compare/v0.1.0...v0.1.1

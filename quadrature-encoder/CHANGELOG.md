# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Please make sure to add your changes to the appropriate categories:

- `Added`: for new functionality
- `Changed`: for changes in existing functionality
- `Deprecated`: for soon-to-be removed functionality
- `Removed`: for removed functionality
- `Fixed`: for fixed bugs
- `Performance`: for performance-relevant changes
- `Security`: for security-relevant changes
- `Other`: for everything else

## [Unreleased]

### Added

- Added support for async-await (via `async` crate feature, enabled by default):
  - Added trailing generic parameter `PollMode` to `IncrementalEncoder<…>` and `IndexedIncrementalEncoder<…>` to support both, blocking and async polling modes:
    - `IncrementalEncoder<…, Blocking>`, which exposes its `fn poll()` as a blocking method.
    - `IndexedIncrementalEncoder<…, Blocking>`, which exposes its `fn poll()` as a blocking method.
    - `IncrementalEncoder<…, Async>`, which exposes its `fn poll()` as an `async` method.
    - `IndexedIncrementalEncoder<…, Async>`, which exposes its `fn poll()` as an `async` method.
  - Added `fn into_async()` and `fn into_blocking()` methods for converting between blocking and non-blocking poll modes:
    - `fn into_async()`, which converts `IncrementalEncoder<…, Blocking>` into its non-blocking equivalent: `IncrementalEncoder<…, Async>`.
    - `fn into_async()`, which converts `IndexedIncrementalEncoder<…, Blocking>` into its non-blocking equivalent: `IndexedIncrementalEncoder<…, Async>`.
    - `fn into_blocking()`, which converts `IncrementalEncoder<…, Async>` back into its blocking equivalent: `IncrementalEncoder<…, Blocking>`.
    - `fn into_blocking()`, which converts `IndexedIncrementalEncoder<…, Async>` back into its blocking equivalent: `IndexedIncrementalEncoder<…, Blocking>`.

### Changed

- Changed generic parameters of `IncrementalEncoder<…>` to `<Mode, Clk, Dt, Steps, T, PM>`, adding trailing `PM` parameter.
- Changed generic parameters of `IndexedIncrementalEncoder<…>` to `<Mode, Clk, Dt, Idx, Steps, T, PM>`, adding trailing `PM` parameter.
- Bumped MSRV from `1.74.1` to `1.75.0`.

### Deprecated

- n/a

### Removed

- n/a

### Fixed

- n/A

### Performance

- n/a

### Security

- n/a

### Other

- n/a

## [0.1.1] - 2024-06-05

### Changed

- Bumped MSRV from `1.74.0` to `1.74.1`

## [0.1.0] - 2024-05-21

Initial release.

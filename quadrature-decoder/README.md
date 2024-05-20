# `quadrature-decoder`

[![Crates.io](https://img.shields.io/crates/v/quadrature-decoder)](https://crates.io/crates/quadrature-decoder)
[![Crates.io](https://img.shields.io/crates/d/quadrature-decoder)](https://crates.io/crates/quadrature-decoder)
[![Crates.io](https://img.shields.io/crates/l/quadrature-decoder)](https://crates.io/crates/quadrature-decoder)
[![docs.rs](https://docs.rs/quadrature-decoder/badge.svg)](https://docs.rs/quadrature-decoder/)

Pure logic-level implementations of [quadrature decoders](https://en.wikipedia.org/wiki/Incremental_encoder#Quadrature_decoder) with support for full-, half- an quad-stepping.

----

## Incremental Decoder

```rust
use quadrature_decoder::{FullStep, IncrementalDecoder};

let mut decoder: IncrementalDecoder<...> = Default::default();

// Update the decoder with pulse trains `a` and `b` and handle the result:
match decoder.update(a, b) {
    Ok(Some(change)) => println!("Change detected: {change:?}."),
    Ok(None) => println!("No change detected."),
    Err(error) => println!("Error detected: {error:?}."),
}

// Or, if you only care about correctly detected changes:
if let Some(change) = decoder.update(a, b).unwrap_or_default() {
    println!("Change detected: {change:?}.")
}

println!("Decoder is at counter: {:?}.", decoder.counter());
```

See the examples directory for a more comprehensive example.

## Indexed Incremental Decoder

An indexed decoder resets its counter whenever a raising edge is detected on the `z` pulse train.

```rust
use quadrature_decoder::{IndexedIncrementalDecoder};

let mut decoder: IndexedIncrementalDecoder<...> = Default::default();

// Update the decoder with pulse trains `a`, `b` and `z` and handle the result:
match decoder.update(a, b, z) {
    Ok(Some(change)) => println!("Change detected: {change:?}."),
    Ok(None) => println!("No change detected."),
    Err(error) => println!("Error detected: {error:?}."),
}

// Or, if you only care about correctly detected changes:
if let Some(change) = decoder.update(a, b, z).unwrap_or_default() {
    println!("Change detected: {change:?}.")
}

println!("Decoder is at counter: {:?}.", decoder.counter());
```

See the examples directory for a more comprehensive example.

## Decoding Strategies

### Full-step Decoding

A full-step decoder is able to detect up to 1 change(s) per quadrature cycle.

```rust
use quadrature_decoder::{FullStep, IncrementalDecoder};

let mut decoder: IncrementalDecoder<FullStep> = Default::default();
```

### Half-step Decoding

A full-step decoder is able to detect up to 2 change(s) per quadrature cycle.

```rust
use quadrature_decoder::{HalfStep, IncrementalDecoder};

let mut decoder: IncrementalDecoder<HalfStep> = Default::default();
```

### Quad-step Decoding

A full-step decoder is able to detect up to 4 change(s) per quadrature cycle.

```rust
use quadrature_decoder::{QuadStep, IncrementalDecoder};

let mut decoder: IncrementalDecoder<QuadStep> = Default::default();
```

## Documentation

Please refer to the documentation on [docs.rs](https://docs.rs/quadrature-decoder).

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](https://www.rust-lang.org/conduct.html),  
and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/regexident/quadrature/tags).

## License

This project is licensed under the [**MPL-2.0**](https://www.tldrlegal.com/l/mpl-2.0) – see the [LICENSE.md](LICENSE.md) file for details.

# `quadrature-encoder`

[![Crates.io](https://img.shields.io/crates/v/quadrature-encoder)](https://crates.io/crates/quadrature-encoder)
[![Crates.io](https://img.shields.io/crates/d/quadrature-encoder)](https://crates.io/crates/quadrature-encoder)
[![Crates.io](https://img.shields.io/crates/l/quadrature-encoder)](https://crates.io/crates/quadrature-encoder)
[![docs.rs](https://docs.rs/quadrature-encoder/badge.svg)](https://docs.rs/quadrature-encoder/)

Hardware-level implementations of drivers for [incremental encoders](https://en.wikipedia.org/wiki/Incremental_encoder) with support for full-, half- an quad-stepping.

----

## Incremental Encoder

```rust
use quadrature_encoder::{FullStep, IncrementalEncoder};

let mut encoder: IncrementalEncoder<...> = Default::new(pin_clk, pin_dt);

// Update the encoder with pulse trains `a` and `b` and handle the result:
match encoder.poll() {
    Ok(Some(movement)) => println!("Movement detected: {movement:?}."),
    Ok(None) => println!("No movement detected."),
    Err(error) => println!("Error detected: {error:?}."),
}

// Or, if you only care about correctly detected movement:
if let Some(movement) = encoder.poll().unwrap_or_default() {
    println!("Movement detected: {movement:?}.")
}

println!("Encoder is at position: {:?}.", encoder.position());
```

See the examples directory for a more comprehensive example.

## Indexed Incremental Encoder

An indexed encoder resets its position whenever a raising edge is detected on the `idx` pin.

```rust
use quadrature_encoder::{IndexedIncrementalEncoder};

let mut encoder: IndexedIncrementalEncoder<...> = Default::new(pin_clk, pin_dt, pin_idx);

// Update the encoder with pulse trains `a`, `b` and `z` and handle the result:
match encoder.poll() {
    Ok(Some(movement)) => println!("Movement detected: {movement:?}."),
    Ok(None) => println!("No movement detected."),
    Err(error) => println!("Error detected: {error:?}."),
}

// Or, if you only care about correctly detected movement:
if let Some(movement) = encoder.poll().unwrap_or_default() {
    println!("Movement detected: {movement:?}.")
}

println!("Encoder is at position: {:?}.", encoder.position());
```

See the examples directory for a more comprehensive example.

## Convenience Aliases

Since the full typename `IncrementalEncoder<Mode, ..., Step, T, PM>` can be quite a mouth-full a couple of convenience type-aliases are provided for the most common use-cases:

### Rotary Encoders

```rust
use quadrature_encoder::{RotaryEncoder, IndexedRotaryEncoder};

let mut encoder = RotaryEncoder::new(pin_clk, pin_dt);
let mut indexed_encoder = IndexedRotaryEncoder::new(pin_clk, pin_dt, pin_idx);
```

### Linear Encoders

```rust
use quadrature_encoder::{LinearEncoder, IndexedLinearEncoder};

let mut encoder = LinearEncoder::new(pin_clk, pin_dt);
let mut indexed_encoder = IndexedLinearEncoder::new(pin_clk, pin_dt, pin_idx);
```

## Async Polling Mode

All encoders support both, blocking as well as non-blocking (i.e. async) polling modes.

To create an async encoder you just have provide the `Async` type parameter:

```rust
let mut async_encoder: RotaryEncoder<_, _, Async> = RotaryEncoder::new(pin_clk, pin_dt);
let mut async_indexed_encoder: IndexedRotaryEncoder<_, _, Async> = IndexedRotaryEncoder::new(pin_clk, pin_dt, pin_idx);
```

Or you can use the `.into_async()` method to convert an existing blocking encoder into a non-blocking one:

```rust
let mut async_encoder = blocking_encoder.into_async();
let mut async_indexed_encoder = blocking_indexed_encoder.into_async();
```

Use the `.into_blocking()` method to convert a non-blocking encoder back into a non-blocking one:

```rust
let mut blocking_encoder = async_encoder.into_blocking();
let mut blocking_indexed_encoder = async_indexed_encoder.into_blocking();
```

## Decoding Strategies

### Full-step Decoding

A full-step encoder is able to detect up to 1 change(s) per quadrature cycle.

```rust
use quadrature_encoder::{FullStep, IncrementalEncoder};

let mut encoder: IncrementalEncoder<_, _, FullStep> = Default::new(...);
```

### Half-step Decoding

A full-step encoder is able to detect up to 2 change(s) per quadrature cycle.

```rust
use quadrature_encoder::{HalfStep, IncrementalEncoder};

let mut encoder: IncrementalEncoder<_, _, HalfStep> = Default::new(...);
```

### Quad-step Decoding

A full-step encoder is able to detect up to 4 change(s) per quadrature cycle.

```rust
use quadrature_encoder::{QuadStep, IncrementalEncoder};

let mut encoder: IncrementalEncoder<_, _, QuadStep> = Default::new(...);
```

## Documentation

Please refer to the documentation on [docs.rs](https://docs.rs/quadrature-encoder).

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](https://www.rust-lang.org/conduct.html),  
and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/regexident/quadrature/tags).

## License

This project is licensed under the [**MPL-2.0**](https://www.tldrlegal.com/l/mpl-2.0) – see the [LICENSE.md](LICENSE.md) file for details.

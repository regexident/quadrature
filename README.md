# `quadrature`

Implementations of a [quadrature decoder](https://en.wikipedia.org/wiki/Incremental_encoder#Quadrature_decoder) and corresponding [incremental encoder](https://en.wikipedia.org/wiki/Incremental_encoder) drivers using [embedded-hal](https://crates.io/crates/embedded-hal) with support for full-, half- an quad-stepping.

----

The project is divided into two separate crates:

## `quadrature-decoder`

Pure logic-level implementations of [quadrature decoders](https://en.wikipedia.org/wiki/Incremental_encoder#Quadrature_decoder) with support for full-, half- an quad-stepping.

See the corresponding [`README.md`](/quadrature-decoder/) for more information.

## `quadrature-encoder`

Implementations of hardware-level drivers for [incremental encoders](https://en.wikipedia.org/wiki/Incremental_encoder) with support for full-, half- an quad-stepping.

See the corresponding [`README.md`](/quadrature-decoder/) for more information.

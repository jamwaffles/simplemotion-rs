# SimpleMotionV2 generated Rust bindings

[![CircleCI](https://circleci.com/gh/jamwaffles/simplemotion-rs.svg?style=shield)](https://circleci.com/gh/jamwaffles/simplemotion-rs)
[![Crates.io](https://img.shields.io/crates/v/simplemotion-sys.svg)](https://crates.io/crates/simplemotion-sys)
[![Docs.rs](https://docs.rs/simplemotion-sys/badge.svg)](https://docs.rs/simplemotion-sys)
[![Liberapay](https://img.shields.io/badge/donate-liberapay-yellow.svg)](https://liberapay.com/jamwaffles)

Provides unsafe, generated bindings to the C [SimpleMotionV2](https://github.com/GraniteDevices/SimpleMotionV2) interface.

For a safe, high-level interface, see the [`simplemotion`](https://crates.io/crates/simplemotion) crate.

```bash
cargo add simplemotion-sys
```

Please consider [becoming a sponsor](https://github.com/sponsors/jamwaffles/) so I may continue to maintain this crate in my spare time!

# [Documentation](https://docs.rs/simplemotion-sys)

# Example

More examples can be found in `simplemotion/examples` and `simplemotion-sys/examples`.

# Development

## Setup

[`bindgen`](https://github.com/rust-lang/rust-bindgen) must be set up correctly. Follow the [requirements section of its docs](https://rust-lang.github.io/rust-bindgen/requirements.html).

The simplemotion bindings are included as a Git submodule:

```bash
git submodule init
git submodule update
```

## Build

```bash
cargo build
```

You can also run `./build.sh` to run all the commands that would normally be run in CI.

## Test

```bash
cargo test
```

## Build docs

```bash
cargo doc --open
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

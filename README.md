# oka

[![Crates.io downloads](https://img.shields.io/crates/d/oka)](https://crates.io/crates/oka) [![crates.io](https://img.shields.io/crates/v/oka?style=flat-square)](https://crates.io/crates/oka) [![Build Status](https://github.com/ararog/oka/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/ararog/oka/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/oka) [![Documentation](https://docs.rs/oka/badge.svg)](https://docs.rs/oka/latest/oka) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ararog/oka/blob/main/LICENSE.md)  [![codecov](https://codecov.io/gh/ararog/oka/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/ararog/oka)

oka is a L1 and L2 caching library for rust async applications.

## Features

- Protocol version 13

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oka = "0.1.0"
```

## Configuration

Below you can find instruction on how to configure many logging options availble
on vetis.

### stdout

#### Using API

```rust,ignore

```

#### vetis.yaml

There's no plans to provide this kind of configuration method.

#### Configuration Options

- **log_level**: Log level: TRACE, WARN, DEBUG, INFO, ERROR and FATAL

### Runtime Features

Choose the appropriate runtime feature for your project:

- `tokio` (default): Use with Tokio runtime
- `smol` (default): Use with Tokio runtime

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>

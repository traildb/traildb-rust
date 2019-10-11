# traildb-rust

Rust bindings for [TrailDB](http://traildb.io)

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
traildb = "0.4.0"
```

At the moment there's no documentation, but a good starting point is
`examples/simple.rs` and the tests in `src/lib.rs`



### Build

This binding is statically linked with a specific version of TrailDB. If you want to build it yourself, make sure you've also cloned the TrailDB:

```shell script
git submodule update --init --recursive
```
# martim

A ~experimental~ superior kernel written in Rust

## Setup

### Requirements

* QEMU
* A Rust nightly build
    * e.g. `rustup toolchain install nightly` as
      per [this](https://doc.rust-lang.org/edition-guide/rust-2018/rustup-for-managing-rust-versions.html) page

### Build and Run

To run the kernel in QEMU

```plain
cargo run
```

To run the tests

```plain
cargo test
```

To only build the image but not run it

```plain
cargo bootimage
```
<div align="center">

# martim

[![build](https://github.com/martimos/kernel/actions/workflows/build.yml/badge.svg)](https://github.com/martimos/kernel/actions/workflows/build.yml)
[![lint](https://github.com/martimos/kernel/actions/workflows/lint.yml/badge.svg)](https://github.com/martimos/kernel/actions/workflows/lint.yml)
[![DeepSource](https://deepsource.io/gh/martimos/kernel.svg/?label=active+issues&token=m2Vll4NTzzn-B2R4HkFEc5V7)](https://deepsource.io/gh/martimos/kernel/?ref=repository-badge)

A <strike>experimental</strike> superior kernel written in Rust

[Requirements](#requirements) •
[Build and Run](#build-and-run) •
[Changelog](CHANGELOG.md) •
[Wiki](https://github.com/martimos/kernel/wiki)

</div>

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

#### What else can I do?

```plain
cargo run -- --help
```

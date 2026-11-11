# bmi-example-rust

[![AI Declaration Format](https://img.shields.io/badge/AI_Declaration_Format-validated-blue)](./aidecl.yaml)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.0000000.svg)](https://doi.org/10.5281/zenodo.0000000)

An example of implementing the [Rust specification](https://crates.io/crates/bmi-rust) for the CSDMS
[Basic Model Interface](https://bmi.csdms.io/en/stable/) (BMI).

## Overview

This example implements a BMI for a simple model that solves the two-dimensional
heat equation on a uniform rectangular plate with fixed boundary temperatures.
The model and its BMI are written in Rust. Tests of the BMI are provided.

The repository contains:

- `src/heat.rs`: the heat model.
- `src/lib.rs`: the BMI implementation.
- `src/main.rs`: a sample program that runs the model through its BMI.
- `tests/`: tests of the model's BMI.

## Build/Run

Install a Rust toolchain supporting edition 2024. Cargo downloads the `bmi-rust`
dependency from crates.io automatically.

Build the example and run the tests from this directory:

```sh
cargo build
cargo test
```

## Use

Run the heat model through its BMI with the supplied configuration:

```sh
cargo run -- heat.cfg
```

The program heats the center node to 300 K and prints its temperature at each
time step. Run `cargo run` without an argument to use the default configuration.

Configuration files contain four comma-separated values: diffusivity, end time,
number of columns, and number of rows. See [heat.cfg](heat.cfg) for an example.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development instructions and AI usage
declaration guidance.

## License

Apache-2.0. See [LICENSE](LICENSE).

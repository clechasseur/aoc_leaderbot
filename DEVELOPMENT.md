This guide is meant for people wishing to contribute to this open-source project.
For more information on contributing, see [CONTRIBUTING](CONTRIBUTING.md).

## Prerequisites

### Rust

You need a Rust toolchain to build this project's code and run the tests.
You can install Rust from the [official website](https://www.rust-lang.org/tools/install).
If you already have a version of Rust installed via `rustup` but it's too old, you can update by running

```shell
rustup update
```

### Rust nightly

Certain tools require a Nightly Rust toolset.
If you do not have one installed, you can install one via `rustup` by running

```shell
rustup toolchain install nightly
```

If you already have one installed, but it was too old, it was probably updated earlier when you ran `rustup update` 😉

### Just

[just](https://github.com/casey/just) is a command-line tool to run scripts, a bit like `npm`'s scripts.
It's written in Rust.

This project includes a [justfile](justfile) that makes it easier to run the various tools used for development.
To install `just` via `cargo`, simply run

```shell
cargo install just --locked
```

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), it'll probably be faster to use it instead:

```shell
cargo binstall just
```

You can also install it via various [methods](https://github.com/casey/just#packages).

### Llvm-cov

If you want to run tests with coverage locally, you'll need to install [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov), a code coverage tool for Rust.
You can install it via `cargo`:

```shell
cargo install cargo-llvm-cov --locked
```

You can also install it via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```shell
cargo binstall cargo-llvm-cov
```

### Docker (or equivalent)

This project includes crates that connect to [AWS DynamoDB](https://aws.amazon.com/dynamodb/).
To test this locally, the [`dynamodb-local`](https://hub.docker.com/r/amazon/dynamodb-local) Docker image is used to run a local, API-compatible DynamoDB service.
To run this container, you will need either Docker or an equivalent containerization engine (like [Podman](https://podman.io/)).

The easiest way to run containers locally is to use [Docker Desktop](https://www.docker.com/products/docker-desktop/).
It is free to use for non-commercial use.

Whatever tool you install, make sure it also supports [Docker Compose](https://docs.docker.com/compose/).

#### Linux

## Development

### Running the local DynamoDB

Both the tests and the bot code can connect to a local DynamoDB instance, running via [`dynamodb-local`](https://hub.docker.com/r/amazon/dynamodb-local).
If tests are executed using the `just` recipe (see below), the DynamoDB container will be started and stopped automatically.
To speed things up, it's also possible to start the container in advance - if it is already running, the tests won't try to start it again.

To start or stop the local DynamoDB container, you can use

```shell
just dynamo
```

The command is a toggle - it detects whether the container is running and starts or stops it as appropriate.

### Running the tests

In order to run all tests, you can use

```shell
just test
```

Any new feature or bug fix would need new tests to validate.
Make sure all tests pass before submitting a PR.

### Linting

Before submitting a PR, make sure `rustfmt` and `clippy` are happy.
To tidy up your code before committing, simply run

```shell
just tidy
```

Required checks will not pass if either of those report issues.

### Code coverage

This project's [code coverage settings](codecov.yml) are pretty stringent.
To validate this locally, you can run

```shell
just llvm-cov
```

Make sure coverage is at the required level before submitting a PR.

### Generating documentation

All public symbols in the project need to be documented, otherwise checks won't pass.
To validate this, you can generate docs locally by running

```shell
just doc
```

Make sure any new public symbol is documented before submitting a PR.

## Questions?

If any part of this documentation is unclear, please open a [new issue](https://github.com/clechasseur/aoc_leaderbot/issues/new/choose) so it can be fixed.

# zentime-rs

[![Release](https://github.com/on3iro/zentime-rs/actions/workflows/release.yaml/badge.svg)](https://github.com/on3iro/zentime-rs/actions/workflows/release.yaml)
[![docs.rs](https://img.shields.io/docsrs/zentime-rs/latest?label=docs)](https://docs.rs/zentime-rs/latest/zentime_rs/)

# Table of Contents

- [zentime-rs](#zentime-rs)
- [Table of Contents](#table-of-contents)
  - [Installation](#installation)
    - [Homebrew](#homebrew)
    - [Cargo](#cargo)
  - [Configuration](#configuration)

A simple terminal based pomodoro/productivity timer written in Rust.

## Installation

> NOTE: The timer has currently only been tested on Mac, but should work fine on linux and windows as well.

### Homebrew

```
brew tap install on3iro/zentime
brew install zentime
```

### Cargo

```
cargo install zentime-rs
```

## Configuration

The default location for the configuration file is `/home/<user>/.config/zentime/zentime.toml`.
To get an overview of available configuration options please have a look at the [example configuration](./zentime.example.toml).

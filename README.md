# zentime-rs

# Table of Contents

- [zentime-rs](#zentime-rs)
- [Table of Contents](#table-of-contents)
  - [Installation](#installation)
  - [Configuration](#configuration)

A simple terminal based pomodoro/productivity timer written in Rust.

## Installation

You can either clone the repo and build the timer yourself, or install the brew tap, if you are on OSX:

```
brew tap install on3iro/zentime
brew install zentime
```

> NOTE: The timer has currently only been tested on Mac, but should work fine on linux and windows as well.
> However you to create those builds yourself for now. (requires the rust toolchain and cargo)
> I will probably publish this to crates.io in the near future, which will make the process slightly easier.

## Configuration

The default location for the configuration file is `/home/<user>/.config/zentime/zentime.toml`.
To get an overview of available configuration options please have a look at the [example configuration](./zentime.example.toml).

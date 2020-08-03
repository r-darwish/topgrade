![Topgrade](doc/topgrade.png)

[![Travis](https://api.travis-ci.org/r-darwish/topgrade.svg?branch=master)](https://travis-ci.org/r-darwish/topgrade)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/github/r-darwish/topgrade?svg=true)](https://ci.appveyor.com/project/r-darwish/topgrade)
![GitHub release](https://img.shields.io/github/release/r-darwish/topgrade.svg)
[![Crates.io](https://img.shields.io/crates/v/topgrade.svg)](https://crates.io/crates/topgrade)
[![AUR](https://img.shields.io/aur/version/topgrade.svg)](https://aur.archlinux.org/packages/topgrade/)
![homebrew](https://img.shields.io/homebrew/v/topgrade.svg)

![Demo](doc/screenshot.gif)

Keeping your system up to date mostly involves invoking more than a single package manager. This
usually results in big shell one-liners saved in your shell history. Topgrade tries to solve this
problem by detecting which tools you use and run their appropriate package managers.

## Installation
Arch Linux users can use the [AUR](https://aur.archlinux.org/packages/topgrade/) package.

On NixOS, use the `topgrade` package in `nixpkgs`.

macOS users can install topgrade via [Homebrew](https://brew.sh/) or [MacPorts](https://www.macports.org/install.php).

Other systems users can either use `cargo install` or use the compiled binaries from the release
page. The compiled binaries contain a self-upgrading feature.

Topgrade isn't guaranteed to work on Rust versions older than the latest stable release. If you
intend to install Topgrade using Cargo then you should either install Rust using rustup or use a
distribution which ships the latest version of Rust, such as Arch Linux.

## Usage
Just run `topgrade`. See [the wiki](https://github.com/r-darwish/topgrade/wiki/Step-list) for the list of things Topgrade supports

## Customization
See `config.example.toml` for an example configuration file.

### Configuration path

The configuration should be placed in the following paths depending by the operating system:

* **macOS** - `~/.config/topgrade.toml`
* **Windows** - `%APPDATA%/topgrade.toml`
* **Other Unix systems** - `~/.config/topgrade.toml`

## Remote execution
You can specify a key called `remote_topgrades` in the configuration file. This key should contain a
list of hostnames that have topgrade installed on them. Topgrade will execute Topgrades on these
remote hosts. To limit the execution only to specific hosts use the `--remote-host-limit` parameter.

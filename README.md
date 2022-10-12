![Topgrade](doc/topgrade.png)

# No longer maintained

This repository is no longer maintained. An effort was made by the community to keep maintaining the project at https://github.com/topgrade-rs/topgrade. I am not involved in this effort nor do I know the people behind it, so I encourage you to inspect their work before using the fork.

I'm not the owner of the packages that ship Topgrade for various package managers, so their maintainers will need to decide what to do. I'm only the owner of the package in creates.io, which will no longer be updated.

[![Travis](https://api.travis-ci.org/r-darwish/topgrade.svg?branch=master)](https://travis-ci.org/r-darwish/topgrade)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/github/r-darwish/topgrade?svg=true)](https://ci.appveyor.com/project/r-darwish/topgrade)
![GitHub release](https://img.shields.io/github/release/r-darwish/topgrade.svg)
[![Crates.io](https://img.shields.io/crates/v/topgrade.svg)](https://crates.io/crates/topgrade)
[![AUR](https://img.shields.io/aur/version/topgrade.svg)](https://aur.archlinux.org/packages/topgrade/)
![homebrew](https://img.shields.io/homebrew/v/topgrade.svg)

![Demo](doc/screenshot.gif)

Keeping your system up to date usually involves invoking multiple package managers.
This results in big, non-portable shell one-liners saved in your shell.
To remedy this, _topgrade_ detects which tools you use and runs the appropriate commands to update them.

## Installation
- Arch Linux: [AUR](https://aur.archlinux.org/packages/topgrade/) package.
- NixOS: _topgrade_ package in `nixpkgs`.
- macOS: [Homebrew](https://brew.sh/) or [MacPorts](https://www.macports.org/install.php).

Other systems users can either use `cargo install` or use the compiled binaries from the release page.
The compiled binaries contain a self-upgrading feature.

Topgrade requires Rust 1.51 or above.

## Usage
Just run `topgrade`.
See [the wiki](https://github.com/r-darwish/topgrade/wiki/Step-list) for the list of things Topgrade supports.

## Customization
See `config.example.toml` for an example configuration file.

### Configuration path

The configuration should be placed in the following paths depending by the operating system:

* **Windows** - `%APPDATA%/topgrade.toml`
* **macOS** and **other Unix systems** - `${XDG_CONFIG_HOME:-~/.config}/topgrade.toml`

## Remote execution
You can specify a key called `remote_topgrades` in the configuration file.
This key should contain a list of hostnames that have topgrade installed on them.
Topgrade will use `ssh` to run `topgrade` on remote hosts before acting locally.
To limit the execution only to specific hosts use the `--remote-host-limit` parameter.

# Topgrade
[![Travis](https://api.travis-ci.org/r-darwish/topgrade.svg?branch=master)](https://travis-ci.org/r-darwish/topgrade)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/github/r-darwish/topgrade?svg=true)](https://ci.appveyor.com/project/r-darwish/topgrade)
![GitHub release](https://img.shields.io/github/release/r-darwish/topgrade.svg)
[![Crates.io](https://img.shields.io/crates/v/topgrade.svg)](https://crates.io/crates/topgrade)
[![AUR](https://img.shields.io/aur/version/topgrade.svg)](https://aur.archlinux.org/packages/topgrade/)
![homebrew](https://img.shields.io/homebrew/v/topgrade.svg)

![Alt Text](doc/screenshot.gif)

Keeping your system up to date mostly involves invoking more than a single package manager. This
usually results in big shell one-liners saved in your shell history. Topgrade tries to solve this
problem by detecting which tools you use and run their appropriate package managers.

## Supported Platforms
Topgrade should probably work on whichever platform it can be build. The real question is whether
Topgrade knows that platform and can utilize its unique features, such as the operating system's
package manager. Topgrade is tested on and knows the following platforms:

* Linux
  * Arch based
  * Red Hat based
  * Debian based
  * Clear Linux
  * Gentoo
  * NixOS
  * openSUSE
  * Void
* DragonFly BSD
* FreeBSD
* macOS
* Windows

## Installation
Arch Linux users can use the [AUR](https://aur.archlinux.org/packages/topgrade/) package.

On NixOS, use the `topgrade` package in `nixpkgs`:

```sh
nix-env -iA topgrade
```

macOS users can install topgrade via Homebrew.

Other systems users can either use `cargo install` or use the compiled binaries from the release
page. The compiled binaries contain a self-upgrading feature.

Topgrade isn't guaranteed to work on Rust versions older than the latest stable release. If you
intend to install Topgrade using Cargo then you should either install Rust using rustup or use a
distribution which ships the latest version of Rust, such as Arch Linux.

### Upgrading Topgrade
The precompiled binaries supplied in the Github releases page are compiled with a self upgrade
feature. Topgrade will try to upgrade itself before attempting anything else and will respawn itself
when an update is downloaded. If you choose to install Topgrade in this method it is recommended
that you place the binary in some place which is writable by your user account, such as
`~/.local/bin`.

If you prefer to have Topgrade installed in system-wide manner then it's recommended to either
install it using the OS package manager or `cargo install`. Topgrade will not have the self upgrade
feature but it will keep itself up to date by calling the operating system's package manager.

## Usage
Just run `topgrade`. It will run the following steps:

* Try to self-upgrade if compiled with this feature. Topgrade will respawn itself if it was upgraded.
* **Linux**: Run the system package manager:
  * **Arch based**: Run [yay](https://github.com/Jguer/yay) or fall back to pacman
  * **Redhat based**: Run `yum upgrade` (or `dnf` if present)
  * **Debian based**: Run `apt update && apt dist-upgrade`
  * **Clear Linux**: Run `swupd update`
  * **Gentoo**: Run `layman -s ALL && emerge --sync -q && eix-update && emerge -uDNa world`
  * **openSUSE**: Run `zypper refresh && zypper dist-upgrade`
  * **Void**: Run `xbps-install -Su`
* **Linux**: Run [etc-update](https://dev.gentoo.org/~zmedico/portage/doc/man/etc-update.1.html):
* **DragonFly BSD**: Upgrade and audit packages
* **FreeBSD**: Upgrade and audit packages
* **Unix**: Run `brew update && brew upgrade`. This should handle both Homebrew and Linuxbrew
* **Unix**: Run `nix upgrade-nix && nix --upgrade`.
* **Unix**: Run [Pearl](https://github.com/pearl-core/pearl) `pearl update`.
* **Windows**: Run Topgrade inside WSL.
* **Windows**: Upgrade Powershell modules
* **Windows**: Upgrade all [Chocolatey](https://chocolatey.org/) packages
* **Windows**: Upgrade all [Scoop](https://scoop.sh) packages
* Check if the following paths are tracked by Git. If so, pull them:
  * ~/.emacs.d (Should work whether you use [Spacemacs](http://spacemacs.org/) or a custom configuration)
  * ~/.zshrc
  * ~/.tmux
  * ~/.config/fish
  * ~/.config/nvim
  * ~/.vim
  * ~/.config/openbox
  * ~/.config/bspwm
  * ~/.config/i3
  * Powershell Profile
  * Custom defined paths
* **Unix**: Run [zr](https://github.com/jedahan/zr) update
* **Unix**: Run [zplug](https://github.com/zplug/zplug) update
* **Unix**: Run [oh-my-zsh](https://github.com/robbyrussell/oh-my-zsh) update
* **Unix**: Run [antigen](https://github.com/zsh-users/antigen) update
* **Unix**: Run [fisher](https://github.com/jorgebucaran/fisher)
* **Unix**: Upgrade tmux plugins with [TPM](https://github.com/tmux-plugins/tpm). *Note*: Do not use
  the `-b` flag in your configuration as suggested by the TPM readme.
* Update Rustup by running `rustup update`. This will also attempt to run `rustup self update` when Rustup is installed inside the home directory.
* Run Cargo [install-update](https://github.com/nabijaczleweli/cargo-update)
* Upgrade Emacs packages (You'll get a better output if you have [Paradox](https://github.com/Malabarba/paradox) installed)
* Upgrade [OCaml packages](https://opam.ocaml.org/)
* Upgrade [vcpkg](https://github.com/Microsoft/vcpkg) globally installed packages
* Upgrade [myrepos](https://myrepos.branchable.com/) managed sourcecode repositories
* Upgrade Python packages installed using [pipx](https://github.com/cs01/pipx)
* Upgrade [R globally installed packages](https://github.com/ankane/jetpack)
* Upgrade [stack](https://docs.haskellstack.org/en/stable/README/)
* Upgrade Vim/Neovim packages. Works with the following plugin frameworks:
  * [NeoBundle](https://github.com/Shougo/neobundle.vim)
  * [Vundle](https://github.com/VundleVim/Vundle.vim)
  * [Plug](https://github.com/junegunn/vim-plug)
  * [Dein](https://github.com/Shougo/dein.vim)
  * [Voom](https://github.com/airblade/voom)
* Node
  * Run `yarn global update` if yarn is installed.
  * Run `npm update -g` if NPM is installed and `npm root -g` is a path inside your home directory.
* Run `composer global update` if Composer's home directory is inside the home directory of the
  user. Run `valet install` after.
* Upgrade Atom packages
* Run `gem upgrade --user-install` if `~/.gem` exists
* **Linux**: Update Flatpak packages
* **Linux**: Update snap packages
* **Linux**: Run [fwupdmgr](https://github.com/hughsie/fwupd) to show firmware upgrade. (View
  only. No upgrades will actually be performed)
* **Linux**: Run `rpi-update` to update Raspberry Pi Firmware
* **Linux**: Run [pihole](https://pi-hole.net/) updater
* Run custom defined commands
* Final stage
  * **Linux**: Run [needrestart](https://github.com/liske/needrestart)
  * **Windows**: Run Windows Update (You'll have to install [PSWindowsUpdate](https://marckean.com/2016/06/01/use-powershell-to-install-windows-updates/))
  * **macOS**: Upgrade App Store applications
  * **FreeBSD**: Run `freebsd-upgrade`

## Customization
See `config.example.toml` for an example configuration file.

### Configuration path

The configuration should be placed in the following paths depending by the operating system:

* **macOS** - `~/Library/Preferences/topgrade.toml`
* **Windows** - `%APPDATA%/topgrade.toml`
* **Other Unix systems** - `~/.config/topgrade.toml`

## Remote execution
You can specify a key called `remote_topgrades` in the configuration file. This key should contain a
list of hostnames that have topgrade installed on them. Topgrade will execute Topgrades on these
remote hosts.

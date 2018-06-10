# Topgrade [![Travis](https://api.travis-ci.org/r-darwish/topgrade.svg?branch=master)](https://travis-ci.org/r-darwish/topgrade)

![Alt Text](doc/screenshot.gif)

Keeping your system up to date mostly involves invoking more than a single package manager. This
usually results in big shell one-liners saved in your shell history. Topgrade tries to solve this
problem by detecting which tools you use and invoke their appropriate package managers.

## Installation
Arch Linux users can use the [AUR](https://aur.archlinux.org/packages/topgrade/) package.

Other systems users can either use `cargo install` or use the compiled binaries from the release page.

## Usage
Just invoke `topgrade`. It will invoke the following steps:

* Check if the following paths are tracked by Git. If so, pull them:
  * ~/.emacs.d (Should work whether you use [Spacemacs](http://spacemacs.org/) or a custom configuration)
  * ~/.zshrc
  * [~/.oh-my-zsh](https://github.com/robbyrussell/oh-my-zsh)
  * ~/.tmux

* *Unix*: Invoke [zplug](https://github.com/zplug/zplug) update
* *Unix*: Upgrade tmux plugins with [TPM](https://github.com/tmux-plugins/tpm)
* Invoke Cargo [install-update](https://github.com/nabijaczleweli/cargo-update)
* Upgrade Emacs packages
* Upgrade Vim packages. Works with the following plugin frameworks:
  * [NeoBundle](https://github.com/Shougo/neobundle.vim)
  * [Vundle](https://github.com/VundleVim/Vundle.vim)
  * [Plug](https://github.com/junegunn/vim-plug)
* Upgrade NPM globally installed packages
* Upgrade Atom packages
* *Linux*: Invoke the system package manager:
  * *Arch*: Invoke [yay](https://github.com/Jguer/yay) or fall back to pacman
  * *CentOS/RHEL*: Invoke `yum upgrade`
  * *Fedora* - Invoke `dnf upgrade`
  * *Debian/Ubuntu*: Invoke `apt update && apt dist-upgrade`
* *Linux*: Invoke [fwupdmgr](https://github.com/hughsie/fwupd) to show firmware upgrade. (View only. No upgrades will actually be performed)
* *Linux*: Run [needrestart](https://github.com/liske/needrestart)
* *macOS*: Upgrade [Homebrew](https://brew.sh/) packages
* *macOS*: Upgrade App Store applications

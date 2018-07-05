# Topgrade
[![Travis](https://api.travis-ci.org/r-darwish/topgrade.svg?branch=master)](https://travis-ci.org/r-darwish/topgrade)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/github/r-darwish/topgrade?svg=true)](https://ci.appveyor.com/project/r-darwish/topgrade)
[![Crates.io](https://img.shields.io/crates/v/topgrade.svg)](https://crates.io/crates/topgrade)

![Alt Text](doc/screenshot.gif)

Keeping your system up to date mostly involves invoking more than a single package manager. This
usually results in big shell one-liners saved in your shell history. Topgrade tries to solve this
problem by detecting which tools you use and run their appropriate package managers.

## Installation
Arch Linux users can use the [AUR](https://aur.archlinux.org/packages/topgrade/) package.

Other systems users can either use `cargo install` or use the compiled binaries from the release page.

## Usage
Just run `topgrade`. It will run the following steps:

* Run the system package manager:
  * *Arch*: Run [yay](https://github.com/Jguer/yay) or fall back to pacman
  * *CentOS/RHEL*: Run `yum upgrade`
  * *Fedora* - Run `dnf upgrade`
  * *Debian/Ubuntu*: Run `apt update && apt dist-upgrade`
* *Unix*: Run `brew update && brew upgrade`. This should handle both Homebrew and Linuxbrew
* *Windows*: Upgrade all [Chocolatey](https://chocolatey.org/) packages
* Check if the following paths are tracked by Git. If so, pull them:
  * ~/.emacs.d (Should work whether you use [Spacemacs](http://spacemacs.org/) or a custom configuration)
  * ~/.zshrc
  * [~/.oh-my-zsh](https://github.com/robbyrussell/oh-my-zsh)
  * ~/.tmux
  * ~/.config/fish/config.fish
  * Custom defined paths
* *Unix*: Run [zplug](https://github.com/zplug/zplug) update
* *Unix*: Run [fisherman](https://github.com/fisherman/fisherman) update
* *Unix*: Upgrade tmux plugins with [TPM](https://github.com/tmux-plugins/tpm)
* Update Rustup by running `rustup update`. This will also attempt to run `rustup self update` when Rustup is installed inside the home directory.
* Run Cargo [install-update](https://github.com/nabijaczleweli/cargo-update)
* Upgrade Emacs packages
* Upgrade Vim/Neovim packages. Works with the following plugin frameworks:
  * [NeoBundle](https://github.com/Shougo/neobundle.vim)
  * [Vundle](https://github.com/VundleVim/Vundle.vim)
  * [Plug](https://github.com/junegunn/vim-plug)
* Upgrade NPM globally installed packages
* Upgrade Atom packages
* *Linux*: Update Flatpak packages
* *Linux*: Update snap packages
* *Linux*: Run [fwupdmgr](https://github.com/hughsie/fwupd) to show firmware upgrade. (View
  only. No upgrades will actually be performed)
* Run custom defined commands
* Final stage
  * *Linux*: Run [needrestart](https://github.com/liske/needrestart)
  * *macOS*: Upgrade App Store applications

## Flags
* `-t/--tmux` - Topgrade will launch itself in a new tmux session. This flag has no effect if
  Topgrade already runs inside tmux. This is useful when using topgrade on remote systems.

## Customization
You can place a configuration file at `~/.config/topgrade.toml`. Here's an example:


``` toml
git_repos = [
    "~/dev/topgrade",
]

[pre_commands]
"Emacs Snapshot" = "rm -rf ~/.emacs.d/elpa.bak && cp -rl ~/.emacs.d/elpa ~/.emacs.d/elpa.bak"

[commands]
"Python Environment" = "~/dev/.env/bin/pip install -i https://pypi.python.org/simple -U --upgrade-strategy eager jupyter"
```
* `git_repos` - A list of custom Git repositories to pull
* `pre_commands` - Commands to execute before starting any action. If any command fails, Topgrade
  will not proceed
* `commands` - Custom upgrade steps. If any command fails it will be reported in the summary as all
  upgrade steps are reported, but it will not cause Topgrade to stop.

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

## Installation
Arch Linux users can use the [AUR](https://aur.archlinux.org/packages/topgrade/) package.

macOS users can install topgrade via Homebrew.

Other systems users can either use `cargo install` or use the compiled binaries from the release
page.

Topgrade isn't guaranteed to work on Rust versions older than the latest stable release. If you
intend to install Topgrade using Cargo then you should either install Rust using rustup or use a
distribution which ships the latest version of Rust, such as Arch Linux.

## Usage
Just run `topgrade`. It will run the following steps:

* *Linux*: Run the system package manager:
  * *Arch*: Run [yay](https://github.com/Jguer/yay) or fall back to pacman
  * *CentOS/RHEL*: Run `yum upgrade`
  * *Fedora* - Run `dnf upgrade`
  * *Debian/Ubuntu*: Run `apt update && apt dist-upgrade`
* *Linux*: Run [etc-update](https://dev.gentoo.org/~zmedico/portage/doc/man/etc-update.1.html):
* *Unix*: Run `brew update && brew upgrade`. This should handle both Homebrew and Linuxbrew
* *Windows*: Upgrade Powershell modules
* *Windows*: Upgrade all [Chocolatey](https://chocolatey.org/) packages
* Check if the following paths are tracked by Git. If so, pull them:
  * ~/.emacs.d (Should work whether you use [Spacemacs](http://spacemacs.org/) or a custom configuration)
  * ~/.zshrc
  * [~/.oh-my-zsh](https://github.com/robbyrussell/oh-my-zsh)
  * ~/.tmux
  * ~/.config/fish
  * ~/.config/nvim
  * ~/.vim
  * ~/.config/openbox
  * Powershell Profile
  * Custom defined paths
* *Unix*: Run [zplug](https://github.com/zplug/zplug) update
* *Unix*: Run [fisherman](https://github.com/fisherman/fisherman) update
* *Unix*: Upgrade tmux plugins with [TPM](https://github.com/tmux-plugins/tpm)
* Update Rustup by running `rustup update`. This will also attempt to run `rustup self update` when Rustup is installed inside the home directory.
* Run Cargo [install-update](https://github.com/nabijaczleweli/cargo-update)
* Upgrade Emacs packages (You'll get a better output if you have [Paradox](https://github.com/Malabarba/paradox) installed)
* Upgrade [OCaml packages](https://opam.ocaml.org/)
* Upgrade Vim/Neovim packages. Works with the following plugin frameworks:
  * [NeoBundle](https://github.com/Shougo/neobundle.vim)
  * [Vundle](https://github.com/VundleVim/Vundle.vim)
  * [Plug](https://github.com/junegunn/vim-plug)
* Node
  * Run `yarn global update` if yarn is installed.
  * Run `npm update -g` if NPM is installed and `npm root -g` is a path inside your home directory.
* Upgrade Atom packages
* Run `gem upgrade --user-install` if `~/.gem` exists
* *Linux*: Update Flatpak packages
* *Linux*: Update snap packages
* *Linux*: Run [fwupdmgr](https://github.com/hughsie/fwupd) to show firmware upgrade. (View
  only. No upgrades will actually be performed)
* Run custom defined commands
* Final stage
  * *Linux*: Run [needrestart](https://github.com/liske/needrestart)
  * *Windows*: Run Windows Update (You'll have to install [PSWindowsUpdate](https://marckean.com/2016/06/01/use-powershell-to-install-windows-updates/))
  * *macOS*: Upgrade App Store applications

## Flags
* `-t/--tmux` - Topgrade will launch itself in a new tmux session. This flag has no effect if
  Topgrade already runs inside tmux. This is useful when using topgrade on remote systems.
* `-n/--dry-run` - Print what should be run.
* `--no-system` - Skip the system upgrade phase.
* `--no-git-repos` - Don't pull custom git repositories.
* `--no-emacs` - Don't upgrade Emacs packages or configuration files.

## Customization
You can place a configuration file at `~/.config/topgrade.toml` (on macOS `~/Library/Preferences/topgrade.toml`).. Here's an example:


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

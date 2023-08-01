# Installation

Make sure you have `git` and `openssl` installed when running `release-plz`.

`release-plz` is a rust binary that can be installed in different ways.

## Download prebuilt binary

The latest release is on [GitHub](https://github.com/MarcoIeni/release-plz/releases/latest).

## Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* Run `cargo install release-plz --locked`.

## Docker

Run `docker pull marcoieni/release-plz`.

## Arch Linux

`release-plz` can be installed from the
[community repository](https://archlinux.org/packages/community/x86_64/release-plz/)
using [pacman](https://wiki.archlinux.org/title/Pacman).

* `pacman -S release-plz`

There is also a VCS package available in the AUR
and it can be installed with an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers).
For example:

* `paru -S release-plz-git`

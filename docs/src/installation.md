# Installation

Make sure you have `git` and `openssl` installed when running `release-plz`.

`release-plz` is a rust binary that can be installed in different ways.

## Download prebuilt binary

The latest release is on [GitHub](https://github.com/MarcoIeni/release-plz/releases/latest)

## Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* Run `cargo install release-plz --locked`

## Docker

`docker pull marcoieni/release-plz`

## AUR

`release-plz` can be installed from available
[AUR packages](https://aur.archlinux.org/packages?O=0&SeB=b&K=release-plz&outdated=&SB=n&SO=a&PP=50&submit=Go)
using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example:

* `paru -S release-plz`
* `paru -S release-plz-git` (VCS package)

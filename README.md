# Manuscript

![Application icon](./data/icons/hicolor/scalable/apps/io.sixpounder.Manuscript.svg)

## This is a work in progress

This software is not production ready and it's still in active development.

---

A text editor specifically designed for book writers, authors and storytellers.

## Installation

The easieast way to install is from Flathub.

<a href="https://flathub.org/apps/details/io.sixpounder.Manuscript"><img src="https://flathub.org/assets/badges/flathub-badge-en.png" width="200"/></a>

### Using Gnome Builder

Just clone this repository and hit the play button. Builder 43 also let you one-click install
the application to your device.

### Build from sources

You will need the meson build system and flatpak builder, along with gtk4 and libadwaita devel libraries.

```bash
git clone <this repo> manuscript
cd manuscript
meson build --prefix=/usr/local
ninja -C build
```


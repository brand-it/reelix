[![publish](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml/badge.svg?branch=release)](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

# Read Before Using

## Using MakeMKV for DVD Ripping

This tool relies on [MakeMKV](https://www.makemkv.com/) to handle the DVD ripping and convert your discs into MKV files. Unfortunately, MakeMKV is not free forever — you’ll need to purchase a license to unlock full functionality. I know, it’s a bit of a bummer, especially since the goal here is to make ripping your movie collection for Plex as smooth and cost-free as possible.

At some point, I plan to stop bundling the MakeMKV binaries with this package. When that happens, you’ll be responsible for managing the MKV ripping process on your own. For now, if you want to take full advantage of the built-in ripping feature, go ahead and download MakeMKV and register it.

### 👉 You can buy and register MakeMKV here:

https://www.makemkv.com/buy/

By registering it using their official process, the binaries used by this tool will be properly activated. Once that’s done, you shouldn’t need to open MakeMKV again unless you want to.

Really There is nothing better out there I could find then MakeMKV. The process to get lossless conversion of your movie data this is the best. It is a lot of money to buy but I will leave that up to you to decided if you think it is worth it. In the end this tool will have more feature out side of simply ripping movies for Plex. So you might end up not using it for the Ripping your collection. Might end up using it to manage your Plex Library. I don't know we will see.

## Development

1. `asdf install`

### Upgrade

Commands to upgrade packages and dependencies

```
npm upgrade
```

```
cargo update --manifest-path src-tauri/Cargo.toml
```

### Using Node

```
npm run tauri dev
```

### Using Cargo

```
cargo install tauri-cli --version "^2.0.0" --locked
```

```
cargo tauri dev --config src-tauri/tauri.dev.conf.json
```

### Linux Packages

```
sudo apt install \
  build-essential \
  libglib2.0-dev \
  libcairo2-dev \
  libgdk-pixbuf2.0-dev \
  libatk1.0-dev \
  libgtk-3-dev \
  libsoup-3.0-dev \
  pkg-config \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  curl \
  wget \
  libappindicator3-dev
```

## Build / Deployment

1. `asdf install`

### Using Node

```
npm run tauri build -- --bundles dmg`
```

### Debug Build Image

```shell
npm run tauri build -- --bundles dmg --debug
```

## Add new Cargo Package

```shell
cargo tauri add tara
```

```shell
cargo add tera --manifest-path src-tauri/Cargo.toml
```

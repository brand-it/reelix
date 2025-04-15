[![publish](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml/badge.svg?branch=release)](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Development

1. `asdf install`

### Using Node

```
npm run tauri dev
```

### Using Cargo

```
cargo install tauri-cli --version "^2.0.0" --locked
cargo tauri dev
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

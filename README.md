[![publish](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml/badge.svg?branch=release)](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)


## Development

1. `asdf install`
2. `npm run tauri dev`


## Build / Deployment

1. `asdf install`
2. `npm run tauri build -- --bundles dmg`


### Debug Build Image
```shell
npm run tauri build -- --bundles dmg --debug
```

## Add new Cargo Package

```shell
cargo add tera --manifest-path src-tauri/Cargo.toml
```

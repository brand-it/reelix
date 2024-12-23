https://github.com/user-attachments/assets/bfdc89c3-582b-44fe-8609-568c67ce92b8



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

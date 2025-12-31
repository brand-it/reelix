[![publish](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml/badge.svg?branch=release)](https://github.com/brand-it/reelix/actions/workflows/tauri-build.yml)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

# Read Before Using

I know your not going to read any of the things below so here is the download link.

- [Download Now](https://brand-it.github.io/reelix)

Now that you have download the file, here is what you need to know to make this tool work.

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
npm outdated
```

```
npm upgrade
```

```
cargo update --manifest-path src-tauri/Cargo.toml
```

### Bumping the Version

When releasing a new version, update the version number in all 4 locations:

```bash
# 1. Update package.json
# Change "version": "0.31.0" to "version": "0.32.0"

# 2. Update src-tauri/Cargo.toml
# Change version = "0.31.0" to version = "0.32.0"

# 3. Update src-tauri/tauri.conf.json
# Change "version": "0.31.0" to "version": "0.32.0"

# 4. Update src-tauri/Cargo.lock
# Change version = "0.31.0" to version = "0.32.0"
```

All four files must have matching version numbers for builds to work correctly across all platforms (macOS, Linux, Windows).

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

Found this helps with development on linux

```
env WEBKIT_DISABLE_DMABUF_RENDERER=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 cargo tauri
 dev --config src-tauri/tauri.linux.dev.conf.json
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

## Develpment Problems Issues & Solutions

Problem

```
cargo tauri dev --config src-tauri/tauri.dev.conf.json
No version is set for command cargo-tauri
Consider adding one of the following versions in yor config file at /Users/brandit/apps/reelix/.tool-versions
rust 1.83.0
```

Solution

```
cargo install tauri-cli --locked --version "^2"
```

### AutoComplete

There is a titles.txt file that has all the possible auto complete for movies. I used SQLlite to pre process the data and then just copy the text into the file. Removed the follow characters ":\, and double spaces. Just clean it up, updating this file ever so often is a good idea but even if we don't it has so much info it can general provide good suggestions for most movies except for new movies. It some times suggest things that don't exist but that is fine. recommend download new movies from the internet as I find them.

# Macro Debugging

Good luck, askama makes micro debugging a pain so best solution I found is delete a single line rerun the thing and just keep checking. This is the worst way to do it but it will get you a answer eventually.

```
error[E0599]: the method `askama_auto_escape` exists for reference `&&AutoEscaper<'_, Option<u32>, Html>`, but its trait bounds were not satisfied
   --> src/templates/seasons.rs:16:10
    |
16  | #[derive(Template)]
    |          ^^^^^^^^ method cannot be called on `&&AutoEscaper<'_, Option<u32>,
```

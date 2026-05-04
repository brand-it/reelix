# Repository Guidelines

## Project Overview

**Reelix** is a cross-platform desktop application for ripping movies and TV shows from optical discs (DVD/Blu-ray) and automatically uploading them to FTP servers for Plex media libraries. Built with **Tauri 2** (Rust backend) and **Hotwire** (Stimulus + Turbo for frontend), it integrates with MakeMKV for disc ripping and **Reelix Manager** (custom GraphQL API) for metadata and upload management.

**Key Features:**
- Optical disc detection and ripping via MakeMKV
- Reelix Manager integration for movie/TV show metadata and upload tracking
- FTP upload with progress tracking
- Auto-rip functionality for discs
- Cross-platform (macOS, Linux, Windows)

---

## Architecture & Data Flow

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (Webview)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Stimulus   │  │    Turbo     │  │   Bootstrap 5    │  │
│  │  Controllers │  │   (SPA-like) │  │     (UI)         │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │ Tauri IPC (window.__TAURI__)
┌──────────────────────────┴──────────────────────────────────┐
│                     Backend (Rust)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Commands   │  │    State     │  │    Services      │  │
│  │  (FTP, Reelix Manager, etc)│  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│                      │              │                       │
│              ┌───────┴───────┐      │                       │
│              │  Askama       │      │                       │
│              │  Templates    │      │                       │
│              └───────────────┘      │                       │
└─────────────────────────────────────┴───────────────────────┘
```

### Data Flow Pattern

1. **User Action** → Stimulus controller captures event
2. **IPC Call** → `turboInvoke(command, args)` invokes Rust via Tauri IPC
3. **Rust Command** → Processes request, accesses `AppState` for shared state
4. **Template Rendering** → Askama template renders HTML wrapped in `<turbo-stream>`
5. **Turbo Response** → Frontend receives HTML, Turbo processes stream actions
6. **DOM Update** → Turbo morphs DOM elements based on stream targets

### Key Modules

#### Rust Backend (`src-tauri/src/`)

| Module | Purpose |
|--------|---------|
| `commands/` | Tauri IPC handlers (disk, rip, auth, settings, general) |
| `state/` | Application state management (`AppState`, `FtpConfig`, etc.) |
| `services/` | Business logic (FTP upload, Reelix Manager API, MakeMKV, version checking) |
| `models/` | Data structures (optical disk info, MKV, title info) |
| `templates/` | Askama template definitions and rendering |
| `the_movie_db/` | Reelix Manager GraphQL client (models compatible with TMDB schema) |

#### Frontend (`src/`)

| Directory | Purpose |
|-----------|---------|
| `javascripts/controllers/` | Stimulus controllers for UI interactions |
| `stylesheets/` | SCSS stylesheets |
| `index.js` | Main entry point |

---

## Key Directories

```
reelix/
├── src/                          # Frontend assets
│   ├── javascripts/
│   │   └── controllers/          # Stimulus controllers
│   ├── stylesheets/              # SCSS files
│   └── index.js                  # Entry point
├── src-tauri/                    # Rust backend (Tauri)
│   ├── src/
│   │   ├── commands/             # Tauri command handlers
│   │   ├── state/                # State management
│   │   ├── services/             # Business logic
│   │   ├── models/               # Data structures
│   │   ├── templates/            # Askama template Rust code
│   │   └── the_movie_db/         # Reelix Manager GraphQL client
│   ├── templates/                # Askama HTML templates
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri*.conf.json          # Platform configs
├── .github/instructions/         # Development guidelines
├── scripts/                      # Build/utility scripts
└── webpack.config.js             # Frontend bundling
```

---

## Development Commands

### Prerequisites

```bash
# Install asdf plugins and versions
asdf install

# Linux dependencies
sudo apt install build-essential libglib2.0-dev libcairo2-dev \
  libgdk-pixbuf2.0-dev libatk1.0-dev libgtk-3-dev libsoup-3.0-dev \
  pkg-config libssl-dev libwebkit2gtk-4.1-dev curl wget libappindicator3-dev
```

### Build Commands

```bash
# Frontend development (watch mode)
npm run watch        # or: make watch
npm run dev          # alias for watch

# Frontend production build
npm run build        # or: make build

# Full Tauri development (platform-specific)
cargo tauri dev --config src-tauri/tauri.linux.dev.conf.json   # Linux
cargo tauri dev --config src-tauri/tauri.macos.conf.json       # macOS
cargo tauri dev                                               # Windows

# Using Makefile
make tauri-dev          # Linux dev with env vars
make tauri-build        # Production build
```

### Code Quality (REQUIRED before commit)

```bash
# Quick syntax/type check
cargo check --manifest-path src-tauri/Cargo.toml

# Run tests
cargo test --manifest-path src-tauri/Cargo.toml

# Linting
cargo clippy --manifest-path src-tauri/Cargo.toml

# Auto-fix clippy suggestions (run before commit to catch issues clippy can fix)
make style

# All validation
make validate   # Runs check + test + clippy
```

### Version Management

```bash
# Bump version (update package.json, Cargo.toml, tauri.conf.json)
make bump TYPE=major   # or: minor, bug

# Manual upgrade
npm outdated && npm upgrade
cargo update --manifest-path src-tauri/Cargo.toml
```

---

## Code Conventions & Common Patterns

### Rust Patterns

#### State Management

All shared state lives in `AppState` (thread-safe with `Arc<Mutex<>>` or `Arc<RwLock<>>`):

```rust
pub struct AppState {
    pub ftp_config: Arc<Mutex<FtpConfig>>,
    pub optical_disks: Arc<RwLock<Vec<Arc<RwLock<OpticalDiskInfo>>>>>,
    pub selected_optical_disk_id: Arc<RwLock<Option<DiskId>>>,
    // ...
}
```

Access state in commands via `CommandArg`:

```rust
#[tauri::command]
pub fn some_command(app: AppHandle, arg: String) -> Result<String, Error> {
    let state = app.state::<AppState>();
    let disks = state.optical_disks.read()?;
    // ...
}
```

#### Command Structure

Commands are organized by domain in `src/commands/`:

```rust
// commands.rs
#[macro_export]
macro_rules! all_commands {
    () => {
        tauri::generate_handler!(
            $crate::commands::disk::selected_disk,
            $crate::commands::rip::rip_movie,
            // ...
        )
    };
}
```

#### Error Handling

Use `Result<T, crate::standard_error::Error>` for commands. Errors are serialized to frontend.

### Frontend Patterns

#### Stimulus Controllers

All controllers extend `@hotwired/stimulus`:

```javascript
import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["movieId", "link"];
  
  rip(event) {
    event.preventDefault();
    turboInvoke("rip_movie", {
      diskId: parseInt(event.currentTarget.dataset.diskId),
      titleId: parseInt(event.currentTarget.dataset.titleId),
    });
  }
}
```

Registration in `controllers/index.js`:

```javascript
import RipOneController from "./rip_movie_controller.js";
application.register("rip-movie", RipOneController);
```

#### Turbo Invoke Pattern

All backend calls go through `turboInvoke()`:

```javascript
window.turboInvoke = async function(command, commandArgs) {
  const tauriResponse = await window.__TAURI__.core.invoke(command, commandArgs);
  window.processTurboResponse(tauriResponse);
  return new Response(tauriResponse, { status: 200 });
};
```

Response processing handles `<turbo-stream>` elements:

```javascript
window.processTurboResponse = function(turboResponse) {
  const template = document.createElement("template");
  template.innerHTML = turboResponse.trim();
  const streams = template.content.querySelectorAll("turbo-stream");
  
  for (const stream of streams) {
    Turbo.renderStreamMessage(stream.outerHTML);
  }
};
```

### Template Patterns (Askama + Turbo)

**CRITICAL:** All templates must return Turbo Stream wrappers, never raw HTML.

#### Template Structure

```
templates/jobs/
├── container.html       # Full HTML structure (used inline)
├── item.html            # Single item HTML
├── update.turbo.html    # Turbo wrapper for container
└── item.turbo.html      # Turbo wrapper for single item
```

#### Rust Template Definition

```rust
use askama::Template;
use crate::templates::InlineTemplate;

#[derive(Template)]
#[template(path = "jobs/container.html")]
pub struct JobsContainer<'a> {
    pub jobs: &'a [JobItem<'a>],
}

impl<'a> JobsContainer<'a> {
    pub fn dom_id(&self) -> &'static str {
        "jobs-container"
    }
}

#[derive(Template)]
#[template(path = "jobs/update.turbo.html")]
pub struct JobsUpdate<'a> {
    pub container: &'a JobsContainer<'a>,
}

pub fn render_jobs(jobs: &[JobItem]) -> Result<String, Error> {
    let container = JobsContainer { jobs };
    let template = JobsUpdate { container: &container };
    crate::templates::render(template)
}
```

#### HTML Templates

`container.html` - Contains ALL HTML structure:
```html
<div id="{{ container.dom_id() }}">
  {% if jobs.is_empty() %}
  <div class="empty-state">No jobs</div>
  {% else %} {% for job in jobs %} {{ job.render_html() | safe }} {% endfor %} {% endif %}
</div>
```

`update.turbo.html` - ONLY Turbo wrapper:
```html
<turbo-stream action="replace" method="morph" target="{{ container.dom_id() }}">
  <template> {{ container.render_html() | safe }} </template>
</turbo-stream>
```

**Key Rules:**
- Use `{{ container.dom_id() }}` for targets, never hardcode IDs
- `container.html` has all HTML; `update.turbo.html` is minimal wrapper
- Always use `| safe` filter with `render_html()`
- Functions return Turbo-wrapped responses only

---

## Important Files

### Configuration

| File | Purpose |
|------|---------|
| `src-tauri/Cargo.toml` | Rust dependencies, version |
| `src-tauri/tauri.conf.json` | Default Tauri config |
| `src-tauri/tauri.linux.dev.conf.json` | Linux dev overrides |
| `src-tauri/tauri.macos.conf.json` | macOS config |
| `package.json` | Frontend deps, scripts, version |
| `webpack.config.js` | Frontend bundling |

### Entry Points

| File | Purpose |
|------|---------|
| `src-tauri/src/main.rs` | Rust app entry |
| `src-tauri/src/lib.rs` | Tauri setup, state management |
| `src/index.js` | Frontend entry |
| `src/javascripts/turbo.js` | Turbo/Stimulus bootstrap |

### Documentation

| File | Purpose |
|------|---------|
| `.github/instructions/tauri-cargo-development-process.instructions.md` | Dev workflow |
| `.github/instructions/turbo-templates-pattern.instructions.md` | Template patterns |
| `.github/instructions/the-movie-db-api-usage.instructions.md` | Reelix Manager API (legacy filename) |
| `.github/instructions/makemkvcon-usage-instructions.instructions.md` | MakeMKV CLI |

---

## Runtime/Tooling Preferences

### Package Manager

**npm** (specified in `packageManager: "npm@1.22.22"`)

### Runtime

- **Frontend:** Node.js (for build tools), Webview at runtime
- **Backend:** Rust 2021 edition, Tauri 2
- **Version Management:** asdf (see `.tool-versions`)

### Build Tools

- **Frontend:** Webpack 5 + Sass + HtmlWebpackPlugin
- **Backend:** Cargo + Tauri CLI
- **Templates:** Askama (compile-time template engine)

### Platform-Specific Configs

Always use platform-specific config for development:
- Linux: `tauri.linux.dev.conf.json`
- macOS: `tauri.macos.conf.json`
- Windows: default `tauri.conf.json`

---

## Testing & QA

### Rust Testing

```bash
# Run all tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run with output
cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture
```

Test framework: Rust's built-in `#[test]` with `wiremock` for HTTP mocking.

### Code Quality

```bash
# Linting (REQUIRED)
cargo clippy --manifest-path src-tauri/Cargo.toml

# Format (if using rustfmt)
cargo fmt --manifest-path src-tauri/Cargo.toml
```

### Frontend Testing

No dedicated test framework. Manual testing via:
```bash
npm run watch    # Hot-reload development
```

### CI/CD

GitHub Actions handles cross-platform builds (see `.github/workflows/tauri-build.yml`).

---

## External Integrations

### Reelix Manager API

Custom GraphQL API for metadata and upload management.
- Base URL: Configured via `AppState.manager_host`
- Auth: OAuth2 device flow with Bearer token (`Authorization: Bearer <TOKEN>`)
- Endpoints: `/graphql` for queries, `/oauth/authorize_device`, `/oauth/token`
- Client ID: `reelix-client`
- Scopes: `search upload`

Key GraphQL operations:
- `searchMulti(query, page)` - Search movies and TV shows
- `movie(id)` - Get movie details with video blobs
- `tv(id)` - Get TV show details with seasons
- `season(tvId, seasonNumber)` - Get season with episodes

OAuth2 Device Flow:
1. Call `/oauth/authorize_device` POST with `{client_id, scope}`
2. User visits `verification_uri_complete` and enters `user_code`
3. Poll `/oauth/token` with `device_code` until authorized (check `interval`)
4. Store `access_token` in `AppState.manager_token` for subsequent requests

**Resumable Uploads (tus protocol + finalize):**
1. POST `/files` with `Upload-Length` and base64-encoded `Upload-Metadata: filename`
2. PATCH `/files/:uid` in chunks with `Upload-Offset` header (resumable)
3. HEAD `/files/:uid` to check progress after interruptions
4. DELETE `/files/:uid` to abort
5. POST `/graphql` mutation `finalizeUpload(input: {uploadId, tmdbId, mediaType, seasonNumber?, episodeNumber?})`

Upload constraints:
- Max file size: 100 GB
- Incomplete uploads expire after 48 hours of inactivity
- `finalizeUpload` moves file from `tmp/tus_uploads` to media library
- TV uploads require `seasonNumber` and `episodeNumber`
### MakeMKV

CLI tool for disc ripping. Key command:
```bash
makemkvcon -r --cache=16 mkv disc:0 all /output/path
```

### FTP

Uses `suppaftp` crate for uploads. Configuration stored in `AppState.ftp_config`.

---

## Common Tasks

### Adding a New Command

1. Create function in `src/commands/<domain>.rs`
2. Add to `all_commands!()` macro in `src/commands.rs`
3. Call from frontend via `turboInvoke("<command_name>", args)`

### Adding a New Template

1. Create template files in `src-tauri/templates/<feature>/`
2. Define Rust structs in `src-tauri/src/templates/<feature>.rs`
3. Implement `dom_id()` method for each template struct
4. Use `#[derive(Template)]` with `#[template(path = "...")]`

### Updating Version

Use `make bump TYPE=major|minor|bug` to update all three version files simultaneously.

---

## Troubleshooting

### Template Compilation Errors

Askama templates compile during `cargo check`. Errors indicate syntax issues or type mismatches in template files.

### Dev Server Not Starting

1. Verify correct config file for your OS
2. Check no other instance is running
3. Ensure all dependencies installed (`cargo check` passes)

### Linux Display Issues

```bash
env WEBKIT_DISABLE_DMABUF_RENDERER=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 \
  cargo tauri dev --config src-tauri/tauri.linux.dev.conf.json
```

### Changes Not Reflecting

- Rust: Restart dev server or manual refresh (Ctrl+R)
- Frontend: Webpack watch should hot-reload

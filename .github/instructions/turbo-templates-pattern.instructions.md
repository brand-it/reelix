---
applyTo: "src-tauri/src/templates/**, src-tauri/templates/**"
---

# Turbo Templates Pattern & Guidelines

This document defines the standard pattern for implementing templates with Turbo Stream updates in the Reelix application.

## Core Principle

**Only render Turbo objects. Never render raw HTML.** All HTML rendering must be wrapped in turbo-stream actions to work correctly in this application.

## Template Organization

Each feature should have a dedicated template module with the following structure:

```
templates/
└── feature_name/
    ├── container.html         # Main HTML structure with inline item rendering
    ├── item.html             # Individual item/card HTML
    ├── item.turbo.html       # Turbo-wrapped single item update
    └── update.turbo.html     # Turbo-wrapped full container update
```

## Rust Module Structure

Create a corresponding module file: `src/templates/feature_name.rs`

```rust
use askama::Template;
use crate::templates::InlineTemplate;

#[derive(Template)]
#[template(path = "feature_name/container.html")]
pub struct FeatureContainer<'a> {
    pub items: &'a [FeatureItem<'a>],
}

impl<'a> FeatureContainer<'a> {
    pub fn dom_id(&self) -> &'static str {
        "feature-container"
    }
}

#[derive(Template)]
#[template(path = "feature_name/item.html")]
pub struct FeatureItem<'a> {
    pub data: &'a YourData,
}

impl<'a> FeatureItem<'a> {
    pub fn dom_id(&self) -> String {
        format!("item-{}", self.data.id)
    }
}

#[derive(Template)]
#[template(path = "feature_name/update.turbo.html")]
pub struct FeatureUpdate<'a> {
    pub container: &'a FeatureContainer<'a>,
}

#[derive(Template)]
#[template(path = "feature_name/item.turbo.html")]
pub struct FeatureItemTurbo<'a> {
    pub item: &'a FeatureItem<'a>,
}

pub fn render_container(items: &[FeatureItem]) -> Result<String, crate::templates::Error> {
    let container = FeatureContainer { items };
    let template = FeatureUpdate { container: &container };
    crate::templates::render(template)
}

pub fn render_item(data: &YourData) -> Result<String, crate::templates::Error> {
    let item = FeatureItem { data };
    let template = FeatureItemTurbo { item: &item };
    crate::templates::render(template)
}
```

## HTML Template Files

### container.html

**Purpose:** Display all items. Contains ALL HTML structure.

```html
<div id="{{ container.dom_id() }}">
  {% if items.is_empty() %}
  <div class="empty-state">
    <!-- Empty state message -->
  </div>
  {% else %} {% for item in items %} {{ item.render_html() | safe }} {% endfor
  %} {% endif %}
</div>
```

**Rules:**

- Contains complete HTML structure with styling
- Uses `InlineTemplate.render_html()` to render items inline
- NEVER has HTML outside of item rendering
- NEVER splits HTML between this and update.turbo.html

### item.html

**Purpose:** Single item/card HTML structure.

```html
<div class="item" id="{{ item.dom_id() }}">
  <!-- Item content -->
</div>
```

**Rules:**

- Self-contained, standalone item
- **MUST** use `{{ item.dom_id() }}` for the ID (never hardcode the format)
- Has stable, unique ID for targeting (managed by `dom_id()` method)
- No wrapper turbo-stream logic

### update.turbo.html

**Purpose:** Turbo-stream wrapper for full container updates.

```html
<turbo-stream action="replace" method="morph" target="{{ container.dom_id() }}">
  <template> {{ container.render_html() | safe }} </template>
</turbo-stream>
```

**Rules:**

- Contains ONLY the turbo-stream action metadata
- Single line or minimal: `{{ container.render_html() | safe }}`
- NO HTML outside of container rendering
- **MUST** use `{{ container.dom_id() }}` for target (never hardcode)

### item.turbo.html (if needed)

**Purpose:** Turbo-stream wrapper for individual item updates.

```html
<turbo-stream action="replace" method="morph" target="{{ item.dom_id() }}">
  <template> {{ item.render_html() | safe }} </template>
</turbo-stream>
```

**Rules:**

- Wraps single item with turbo-stream action
- **MUST** use `{{ item.dom_id() }}` for target (never hardcode)
- Target ID must match item HTML ID exactly (guaranteed by using `dom_id()`)
- Ensures ID format changes propagate everywhere automatically
- Used when individual items update frequently without full list refresh

## InlineTemplate Pattern

The `InlineTemplate` trait enables calling `render_html()` on template types.

```rust
// In templates/mod.rs or templates/your_module.rs
use crate::templates::InlineTemplate;

// Templates automatically implement InlineTemplate via #[derive(Template)]
// This gives them the render_html() method
{{ item.render_html() | safe }}
```

**Rules:**

- Only use within template files (HTML)
- Always use with `| safe` filter to prevent escaping
- Templates must have `#[derive(Template)]` to support this
- Never call render_html() in Rust - use `crate::templates::render()` instead

## Code Comments Guidelines

Keep code clean and comment-free unless necessary:

❌ **Don't write obvious comments:**

```rust
// Container turbo wrapper - for full list updates
#[derive(Template)]
pub struct FeatureUpdate<'a> {
    pub container: &'a FeatureContainer<'a>,
}

// ⚠️ ONLY store things that respond to render_html()
pub struct FeatureItemTurbo<'a> {
    pub item: &'a FeatureItem<'a>,
}

/// Render a single job item wrapped in turbo-stream for targeted update.
/// Called on individual job progress changes.
pub fn render_item(data: &Data) -> Result<String, Error> {
```

✅ **Write clean code that speaks for itself:**

```rust
#[derive(Template)]
#[template(path = "feature_name/update.turbo.html")]
pub struct FeatureUpdate<'a> {
    pub container: &'a FeatureContainer<'a>,
}

#[derive(Template)]
#[template(path = "feature_name/item.turbo.html")]
pub struct FeatureItemTurbo<'a> {
    pub item: &'a FeatureItem<'a>,
}

pub fn render_item(data: &Data) -> Result<String, crate::templates::Error> {
    let item = FeatureItem { data };
    let template = FeatureItemTurbo { item: &item };
    crate::templates::render(template)
}
```

**Rules:**

- Remove line comments that just restate the code
- Remove doc comments that duplicate what the code clearly shows
- Remove warning/instruction comments like `⚠️` (those are for instructions only)
- Keep comments only when code logic is non-obvious
- Usage examples in docs are OK when the function isn't self-explanatory

**Rule:** Functions should ONLY return Turbo objects, never raw HTML.

```rust
// ✅ CORRECT - Returns turbo-wrapped update
pub fn render_item(data: &Data) -> Result<String, Error> {
    let item = FeatureItem { data };
    let template = FeatureItemTurbo { item: &item }; // Only pass item
    crate::templates::render(template)
}

// ❌ WRONG - Returns raw HTML
pub fn render_item(data: &Data) -> Result<String, Error> {
    let template = FeatureItem { data };
    crate::templates::render(template)  // No turbo wrapper!
}
```

## DOM ID Management

Every template struct that represents a DOM element must implement a `dom_id()` method that returns its stable ID.

```rust
impl<'a> FeatureContainer<'a> {
    pub fn dom_id(&self) -> &'static str {
        "feature-container"
    }
}

impl<'a> FeatureItem<'a> {
    pub fn dom_id(&self) -> String {
        format!("feature-item-{}", self.data.id)
    }
}
```

**Why this matters:**

- **Single source of truth** - ID format defined in one place
- **Easy refactoring** - Change format once, updates everywhere automatically
- **No mismatches** - Template always targets correct ID
- **Type safety** - Compiler ensures IDs are consistent

**Template usage:**

✅ **CORRECT - Uses dom_id():**

```html
<!-- In item.html -->
<div id="{{ item.dom_id() }}">...</div>

<!-- In item.turbo.html -->
<turbo-stream
  action="replace"
  method="morph"
  target="{{ item.dom_id() }}"
></turbo-stream>
```

❌ **WRONG - Hardcoded format:**

```html
<!-- In item.html -->
<div id="feature-item-{{ data.id }}">...</div>

<!-- In item.turbo.html -->
<turbo-stream
  action="replace"
  method="morph"
  target="feature-item-{{ data.id }}"
></turbo-stream>
```

## DOM Structure Requirements

Each container must have a stable target ID via `dom_id()`:

```html
<div id="{{ container.dom_id() }}">
  <!-- Content -->
</div>
```

Items within container must have unique, stable IDs via `dom_id()`:

```html
<div id="{{ item.dom_id() }}" class="item">
  <!-- Item content -->
</div>
```

## State Management

When rendering updates from state changes:

1. **Container update:** Clone job data, create items array, render container turbo
2. **Single item update:** Clone individual job, render item turbo

```rust
// Container update (e.g., when jobs list changes)
pub fn emit_container_update(app_handle: &AppHandle) -> Result<(), Error> {
    let jobs = get_all_jobs();  // Clone data
    let items: Vec<JobsItem> = jobs.iter()
        .map(|job| JobsItem { job })
        .collect();
    let turbo = crate::templates::jobs::render_container(&items)?;
    app_handle.emit("disks-changed", turbo)?;
    Ok(())
}

// Single item update (e.g., when individual job progress changes)
pub fn emit_item_update(app_handle: &AppHandle, job: &Job) -> Result<(), Error> {
    let turbo = crate::templates::jobs::render_item(job)?;
    app_handle.emit("disks-changed", turbo)?;
    Ok(())
}
```

## Common Mistakes to Avoid

❌ **Storing non-template data in Turbo structs**

```rust
#[derive(Template)]
#[template(path = "feature/item.turbo.html")]
pub struct FeatureItemTurbo<'a> {
    pub item: &'a FeatureItem<'a>,
    pub data: &'a YourData,  // WRONG! YourData doesn't respond to render_html()
}

pub fn render_item(data: &YourData) -> Result<String, Error> {
    let item = FeatureItem { data };
    let template = FeatureItemTurbo { item: &item, data };  // WRONG!
    crate::templates::render(template)
}
```

✅ **Only store templates (things that render HTML)**

```rust
#[derive(Template)]
#[template(path = "feature/item.turbo.html")]
pub struct FeatureItemTurbo<'a> {
    pub item: &'a FeatureItem<'a>,  // ONLY this - it responds to render_html()
}

pub fn render_item(data: &YourData) -> Result<String, Error> {
    let item = FeatureItem { data };
    let template = FeatureItemTurbo { item: &item };  // Only pass item
    crate::templates::render(template)
}
```

**Why:** Turbo structs are wrapper-only. They don't contain data - they wrap templates that render HTML. Storing non-template data violates the separation of concerns and leads to HTML duplication.

---

❌ **Splitting HTML between container and turbo files**

```html
<!-- container.html -->
<div id="container">{% for item in items %}...{% endfor %}</div>

<!-- update.turbo.html -->
<div class="extra-html"><!-- WRONG! --></div>
<turbo-stream...></turbo-stream...>
```

✅ **Keep all HTML in container only**

```html
<!-- container.html -->
<div id="container">
  {% if items.is_empty() %}...{% endif %}
  {% for item in items %}...{% endfor %}
</div>

<!-- update.turbo.html -->
<turbo-stream...>{{ container.render_html() }}</turbo-stream>
```

---

❌ **Rendering raw HTML from Rust functions**

```rust
fn get_item_html(data: &Data) -> String {
    FeatureItem { data }.render_html()  // WRONG! No turbo wrapper
}
```

✅ **Always wrap in turbo via render function**

```rust
fn render_item(data: &Data) -> Result<String, Error> {
    let item = FeatureItem { data };
    let template = FeatureItemTurbo { item: &item };
    crate::templates::render(template)
}
```

---

❌ **Using placeholders and separate rendering**

```html
<!-- container.html -->
<div id="job-item-{{ job.id }}" class="placeholder"></div>

<!-- Then render separately... -->
```

✅ **Render items inline**

```html
<!-- container.html -->
{% for item in items %} {{ item.render_html() | safe }} {% endfor %}
```

## Example Implementation

See `src/templates/jobs.rs` and `templates/jobs/` for a complete reference implementation.

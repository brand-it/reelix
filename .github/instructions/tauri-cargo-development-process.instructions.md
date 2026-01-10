---
applyTo: "**"
---

# Tauri Cargo Development Process

This document outlines the standard development workflow for building, testing, and deploying the Reelix application using Tauri and Cargo.

**Agent rule:** After making code changes in this repository, always run `cargo tauri build` and report the result before considering the work complete. This is mandatory for every change.

## Running the Development Server

The application uses different configuration files for each operating system to accommodate platform-specific file paths and settings. Use the appropriate command for your OS:

### Linux Development

```bash
cargo tauri dev --config src-tauri/tauri.linux.conf.json
```

### macOS Development

```bash
cargo tauri dev --config src-tauri/tauri.macos.conf.json
```

**Note:** macOS has different file path requirements, so it uses a dedicated configuration file.

### Windows Development

```bash
cargo tauri dev
```

## Building the Application

Before committing changes or creating pull requests, always build the application to catch compilation errors and warnings early.

### Build Command

```bash
cargo tauri build
```

### Build in Release Mode (Optional for Performance Testing)

```bash
cargo tauri build --release
```

## Code Quality Standards

When making changes to the Rust codebase, follow this process:

1. **Make your changes** to the source code
2. **Run cargo build** to compile the project
3. **Review all compiler warnings** and address them in your solution
4. **Resolve any compilation errors** before proceeding
5. **Test your changes** using the development server for your OS
6. **Only then** commit your changes and land your solution

### Handling Compiler Warnings

Do not ignore compiler warnings. They often indicate:

- Unused code that should be removed
- Potential logic errors
- Performance issues
- API deprecation warnings

Address each warning by either:

- Fixing the underlying issue
- Adding appropriate compiler directives if the warning is intentional (e.g., `#[allow(...)]`)

### Testing Your Changes

After building successfully:

1. Run the dev server for your operating system
2. Test the specific features you modified
3. Verify that no new warnings appear in the build output
4. Check for runtime errors in both the console and application logs

## Working with Askama Templates

Reelix uses [Askama](https://github.com/askama-rs/askama) for server-side template rendering. Askama templates are compiled at build time, providing type safety and performance.

### Template Files Location

Templates are located in:

```
src-tauri/templates/
```

### Askama Template Syntax

Askama uses a Jinja2-like syntax. Common patterns include:

**Variables:**

```html
<div>{{ variable_name }}</div>
```

**Conditionals:**

```html
{% if condition %}
<p>Condition is true</p>
{% endif %}
```

**Loops:**

```html
{% for item in items %}
<div>{{ item }}</div>
{% endfor %}
```

**Template Inheritance:**

```html
{% extends "base.html" %} {% block content %}
<p>Child content</p>
{% endblock %}
```

### Defining Askama Templates in Rust

When using Askama, define templates in your Rust code:

```rust
use askama::Template;

#[derive(Template)]
#[template(path = "my_template.html")]
struct MyTemplate {
    name: String,
    items: Vec<String>,
}
```

### Template Compilation

Askama templates are compiled during the `cargo build` step. If there are template syntax errors or type mismatches:

1. The build will fail with a clear error message
2. The error will indicate the template file and line number
3. Fix the template according to the error message
4. Run `cargo build` again to verify the fix

### Common Askama Issues

**Type Mismatches:**
Ensure that variables passed to templates match the types expected in the template. For example, if the template expects `Vec<String>`, make sure you're passing exactly that type.

**Missing Fields:**
If you remove a field from a template's struct, you must also remove or update the template to not reference that field.

**Path Issues:**
Template paths in `#[template(path = "...")]` are relative to the `templates/` directory.

### Updating Templates

When modifying templates:

1. Make your template changes
2. Update the corresponding Rust struct if needed
3. Run `cargo build`
4. Address any template compilation errors
5. Test the changes in the dev server
6. Verify the rendered output matches expectations

### Askama Resources

For more information about Askama syntax, features, and best practices:

- [Official Askama Repository](https://github.com/askama-rs/askama)
- Template documentation and examples in the repository README
- Check existing templates in `src-tauri/templates/` for reference implementations

## Troubleshooting

### Build Fails with Template Errors

Check the template file syntax in `src-tauri/templates/` and ensure all variables are defined in the corresponding Rust struct.

### Development Server Won't Start

Verify that:

1. All dependencies are installed (`cargo build` completes successfully)
2. You're using the correct config file for your OS
3. No other instance of the application is running on the same ports

### Changes Not Reflected in Dev Server

The dev server auto-reloads on Rust changes, but may need a manual refresh. Try:

1. Manually refreshing the application (Ctrl+R or Cmd+R)
2. Restarting the dev server if changes still don't appear

## Platform-Specific Configuration Files

The application uses these configuration files:

- `src-tauri/tauri.linux.dev.conf.json` - Linux development
- `src-tauri/tauri.macos.conf.json` - macOS (development)
- `src-tauri/tauri.linux.conf.json` - Linux production
- `src-tauri/tauri.windows.conf.json` - Windows production
- `tauri.conf.json` - Default/fallback configuration

When adding platform-specific changes, ensure all relevant configuration files are updated.

# Commit Message Generator

Generates conventional commit messages based on git changes (staged, unstaged, or both).

## Usage

```bash
# Generate commit message for all changes (default)
skill://commit-message

# Generate commit message for staged changes only
skill://commit-message --staged

# Generate commit message for unstaged changes only
skill://commit-message --unstaged

# Generate commit message for all changes
skill://commit-message --all
```

## Commit Types

The generator auto-detects commit type based on file patterns and content:

- **feat:** New features (src/ changes, new files, feature keywords)
- **fix:** Bug fixes (fix/bug/error keywords, error handling)
- **docs:** Documentation changes (.md, .txt, README files)
- **refactor:** Code refactoring (structural changes without new features)
- **chore:** Maintenance (config files, dependencies, tests)
- **style:** Formatting changes (css, scss, style files)
- **test:** Test additions/modifications (test files)

## Output

The generated commit message is piped directly to `git commit -m "<message>"`.

## Examples

```bash
# After making changes and staging them
git add src/javascripts/controllers/rip_movie_controller.js
skill://commit-message --staged

# Changes to documentation
git add README.md
skill://commit-message --staged

# Mixed changes
git add .
skill://commit-message --all
```

## Configuration

No configuration required. The skill auto-detects commit type based on:
- File paths and extensions
- File additions/deletions
- Diff content patterns

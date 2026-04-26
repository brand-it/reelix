#!/bin/bash

# Commit Message Generator
# Generates conventional commit messages based on git changes

set -e

# Default to all changes
SCOPE="all"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --staged)
      SCOPE="staged"
      shift
      ;;
    --unstaged)
      SCOPE="unstaged"
      shift
      ;;
    --all)
      SCOPE="all"
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: skill://commit-message [--staged|--unstaged|--all]"
      exit 1
      ;;
  esac
done

# Get git root
GIT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "Error: Not a git repository"
  exit 1
}

cd "$GIT_ROOT"

# Check for changes
get_changes() {
  case $SCOPE in
    staged)
      git diff --cached --name-status 2>/dev/null || echo ""
      ;;
    unstaged)
      git diff --name-status 2>/dev/null || echo ""
      ;;
    all)
      {
        git diff --cached --name-status 2>/dev/null || echo ""
        git diff --name-status 2>/dev/null || echo ""
      } | sort -u
      ;;
  esac
}

CHANGES=$(get_changes)

if [[ -z "$CHANGES" ]]; then
  echo "Error: No changes found for $SCOPE changes"
  exit 1
fi

# Analyze changes and determine commit type
detect_commit_type() {
  local has_docs=0
  local has_test=0
  local has_style=0
  local has_fix=0
  local has_feat=0
  local has_refactor=0
  local has_chore=0
  
  while IFS=$'\t' read -r status file; do
    [[ -z "$file" ]] && continue
    
    # Check file path patterns
    case "$file" in
      *.md|*.txt|README*|CHANGELOG*|LICENSE)
        has_docs=1
        ;;
      *test*|*spec*|*.test.*|*.spec.*|tests/|__tests__/)
        has_test=1
        ;;
      *.css|*.scss|*.sass|*.less|*.styl)
        has_style=1
        ;;
      *.json|*.yaml|*.yml|.gitignore|.editorconfig|package-lock.json|Cargo.lock)
        has_chore=1
        ;;
      src/|app/|lib/|packages/)
        has_feat=1
        ;;
    esac
    
    # Check if new file (feature indicator)
    if [[ "$status" == "A" ]]; then
      has_feat=1
    fi
    
  done <<< "$CHANGES"
  
  # Priority order: test > docs > style > fix > feat > refactor > chore
  if [[ $has_test -eq 1 && $has_docs -eq 0 && $has_style -eq 0 && $has_feat -eq 0 ]]; then
    echo "test"
  elif [[ $has_docs -eq 1 && $has_test -eq 0 && $has_style -eq 0 && $has_feat -eq 0 ]]; then
    echo "docs"
  elif [[ $has_style -eq 1 && $has_docs -eq 0 && $has_test -eq 0 && $has_feat -eq 0 ]]; then
    echo "style"
  elif [[ $has_feat -eq 1 ]]; then
    echo "feat"
  elif [[ $has_chore -eq 1 && $has_feat -eq 0 && $has_docs -eq 0 && $has_test -eq 0 && $has_style -eq 0 ]]; then
    echo "chore"
  else
    echo "refactor"
  fi
}

# Generate commit message subject
generate_subject() {
  local commit_type=$1
  local files=()
  
  # Get list of changed files (just filenames, not status)
  while IFS=$'\t' read -r status file; do
    [[ -z "$file" ]] && continue
    files+=("$file")
  done <<< "$CHANGES"
  
  # Limit to first 3 files for brevity
  local file_count=${#files[@]}
  local display_files=()
  
  if [[ $file_count -le 3 ]]; then
    display_files=("${files[@]}")
  else
    display_files=("${files[0]}" "${files[1]}" "${files[2]}")
  fi
  
  # Clean up file names for display
  local subject_files=""
  for file in "${display_files[@]}"; do
    # Remove common prefixes and keep just the relevant part
    local clean_file=$(echo "$file" | sed -E "s#^[./]+##" | sed -E "s#^(src/|app/|lib/)##")
    
    if [[ -n "$subject_files" ]]; then
      subject_files="$subject_files, $clean_file"
    else
      subject_files="$clean_file"
    fi
    
    # Truncate if too long
    if [[ ${#subject_files} -gt 50 ]]; then
      subject_files="${subject_files:0:47}..."
      break
    fi
  done
  
  # Handle multiple files
  if [[ $file_count -gt 3 ]]; then
    subject_files="$subject_files (+$((file_count - 3)) more)"
  fi
  
  echo "$commit_type: $subject_files"
}

# Detect commit type
COMMIT_TYPE=$(detect_commit_type)

# Generate subject
SUBJECT=$(generate_subject "$COMMIT_TYPE")

# Create commit
git commit -m "$SUBJECT"

echo "Commit created: $SUBJECT"

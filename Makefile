SHELL := /usr/bin/env bash

.PHONY: help install watch dev build tauri tauri-dev tauri-dev-linux tauri-build check test clippy validate bump bump-major bump-minor bump-bug

help:
	@echo "Available targets:"
	@echo "  make install           # npm install"
	@echo "  make watch             # webpack watch"
	@echo "  make dev               # alias for watch"
	@echo "  make build             # webpack production build"
	@echo "  make tauri ARGS='...'  # npm run tauri -- <args>"
	@echo "  make tauri-dev         # cargo tauri dev (linux dev config)"
	@echo "  make tauri-dev-linux   # cargo tauri dev (linux dev config)"
	@echo "  make tauri-build       # cargo tauri build"
	@echo "  make check             # cargo check"
	@echo "  make test              # cargo test"
	@echo "  make clippy            # cargo clippy"
	@echo "  make validate          # check + test + clippy"
	@echo "  make bump TYPE=...     # TYPE=major|minor|bug"
	@echo "  make bump-major        # bump major"
	@echo "  make bump-minor        # bump minor"
	@echo "  make bump-bug          # bump bug/patch"

install:
	npm install

watch:
	npm run watch

dev: watch

build:
	npm run build

tauri:
	npm run tauri -- $(ARGS)

tauri-dev:
	cargo tauri dev --config src-tauri/tauri.linux.dev.conf.json

tauri-dev-linux: tauri-dev

tauri-build:
	cargo tauri build

check:
	cargo check --manifest-path src-tauri/Cargo.toml

test:
	cargo test --manifest-path src-tauri/Cargo.toml

clippy:
	cargo clippy --manifest-path src-tauri/Cargo.toml

validate: check test clippy

bump:
	@if [ -z "$(TYPE)" ]; then \
		echo "Usage: make bump TYPE=major|minor|bug"; \
		exit 1; \
	fi
	bash scripts/bump-version.sh $(TYPE)

bump-major:
	bash scripts/bump-version.sh major

bump-minor:
	bash scripts/bump-version.sh minor

bump-bug:
	bash scripts/bump-version.sh bug

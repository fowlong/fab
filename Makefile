SHELL := /bin/bash
SHELLFLAGS := -eu -o pipefail -c
.RECIPEPREFIX := >

.PHONY: bootstrap backend\:run web\:dev test golden

bootstrap:
> @command -v rustup >/dev/null 2>&1 || { echo "rustup is required: https://rustup.rs/" >&2; exit 1; }
> @command -v npm >/dev/null 2>&1 || { echo "npm is required: https://nodejs.org/" >&2; exit 1; }
> rustup toolchain install stable --profile minimal --component rustfmt --component clippy
> cargo fetch --manifest-path backend/Cargo.toml
> cd frontend && npm install

backend\:run:
> cargo run --manifest-path backend/Cargo.toml

web\:dev:
> cd frontend && npm run dev -- --host

test:
> cargo test --manifest-path backend/Cargo.toml
> cd frontend && npm run build

golden:
> cargo test --manifest-path backend/Cargo.toml -- --ignored
> cd frontend && npm run build

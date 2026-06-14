# llama-crab — top-level convenience targets.
#
# Most workflows use cargo directly.  This Makefile only adds a couple
# of niceties:
#
#   make                # short alias for `cargo build --workspace`
#   make test           # cargo test --workspace --no-default-features
#   make clippy         # cargo clippy --workspace --all-targets
#   make docs           # cargo doc --workspace --no-deps
#   make models         # download GGUF fixtures used by integration tests
#   make status         # show disk usage of the things `make clean` removes
#   make clean          # full repo clean — removes every build artifact,
#                       # downloaded model, JS dependency, generated doc,
#                       # IDE folder and Cargo cache in the repository
#                       # and all of its subdirectories (asks first)
#
# Run `make help` for the full list.

SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

ROOT := $(shell pwd)

.DEFAULT_GOAL := help

.PHONY: help build test clippy docs fmt models status clean

help: ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; printf "Targets:\n"} \
	     /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2 }' \
	     $(MAKEFILE_LIST)

build: ## cargo build --workspace
	cargo build --workspace

test: ## cargo test --workspace --no-default-features
	cargo test --workspace --no-default-features

clippy: ## cargo clippy --workspace --all-targets -- -D warnings
	cargo clippy --workspace --all-targets -- -D warnings

docs: ## Build rustdoc for the workspace
	cargo doc --workspace --no-deps

fmt: ## cargo fmt --all
	cargo fmt --all

models: ## Download GGUF fixtures used by integration tests
	./scripts/download_models.sh all

status: ## Show disk usage of the things `make clean` would remove
	./scripts/clean.sh --dry-run

clean: ## Wipe every build artifact, model, JS dep, generated doc and cache
	./scripts/clean.sh

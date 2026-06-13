# llama-crab — top-level convenience targets.
#
# Most workflows use cargo directly.  This Makefile only adds a couple
# of niceties:
#
#   make                # short alias for `cargo build --workspace`
#   make test           # cargo test --workspace --no-default-features
#   make clippy         # cargo clippy --workspace --all-targets
#   make docs           # mdBook + rustdoc
#   make models         # download every GGUF used by the examples
#   make quickstart     # download the small Qwen model and run the
#                       # quickstart example end-to-end
#   make clean          # remove target/ + downloaded models (~10-20 GB)
#   make clean-keep     # just remove target/ (keep the models)
#   make clean-all      # remove target/ + models + cargo registry cache
#   make status         # show disk usage of the things `make clean` removes
#
# Run `make help` for the full list.

SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

ROOT := $(shell pwd)

.DEFAULT_GOAL := help

.PHONY: help build test clippy docs fmt models quickstart stateful-chat \
        vision-gemma vision-lfm embedding-search status clean clean-keep \
        clean-all clean-models

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

docs: ## Build mdBook and rustdoc
	cargo doc --workspace --no-deps
	@if [[ -d docs ]]; then (cd docs && mdbook build); fi

fmt: ## cargo fmt --all
	cargo fmt --all

models: ## Download every GGUF model used by the examples
	./scripts/download_models.sh all

quickstart: ## Run the quickstart example end-to-end
	./examples/run.sh quickstart

stateful-chat: ## Run the stateful_chat REPL
	./examples/run.sh stateful_chat

vision-gemma: ## Run the vision example with Gemma 4
	./examples/run.sh vision gemma4

vision-lfm: ## Run the vision example with LFM2.5-VL
	./examples/run.sh vision lfm-vl

embedding-search: ## Run the embedding_search example
	./examples/run.sh embedding_search

status: ## Show disk usage of the things `make clean` would remove
	./scripts/clean.sh --dry-run

clean-keep: ## Remove only target/ (keep the downloaded models)
	./scripts/clean.sh --keep-models

clean-models: ## Remove only the downloaded models
	./scripts/clean.sh --only-models

clean: ## Remove target/ + downloaded models (~10-20 GB)
	./scripts/clean.sh

clean-all: ## Remove target/ + models + cargo registry cache (nuclear)
	./scripts/clean.sh --all

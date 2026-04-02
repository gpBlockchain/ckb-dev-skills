SHELL := /bin/bash

PRETTIER := npx --yes prettier@3.5.3
CSPELL := npx --yes cspell@8.17.5
LYCHEE_VERSION := 0.16.1
LYCHEE_ROOT := .tools
LYCHEE_BIN := $(LYCHEE_ROOT)/bin/lychee
MARKDOWN_FILES := README.md agents/ckb-dev-lead/*.md agents/ckb-core/*.md agents/ckb-contract/*.md agents/ckb-dapp/*.md agents/ckb-fiber/*.md shared/*.md skills/brainstorming/*.md skills/contract-design/*.md commands/*.md

.DEFAULT_GOAL := help

.PHONY: help check docs-check docs-format docs-format-check docs-spell docs-links ensure-lychee

help:
	@echo "Available targets:"
	@echo "  make docs-check        Run markdown format, spelling, and link checks"
	@echo "  make docs-format-check Check Markdown formatting with Prettier"
	@echo "  make docs-format       Rewrite Markdown files with Prettier"
	@echo "  make docs-spell        Run spelling checks with CSpell"
	@echo "  make docs-links        Run link checks with Lychee"

check: docs-check

docs-check: docs-format-check docs-spell docs-links

docs-format-check:
	$(PRETTIER) --check $(MARKDOWN_FILES)

docs-format:
	$(PRETTIER) --write $(MARKDOWN_FILES)

docs-spell:
	$(CSPELL) lint --config .cspell.json $(MARKDOWN_FILES)

docs-links: ensure-lychee
	$(LYCHEE_BIN) --config .lychee.toml --no-progress README.md agents/ shared/ skills/ commands/

ensure-lychee:
	@if [ ! -x "$(LYCHEE_BIN)" ] || ! "$(LYCHEE_BIN)" --version | grep -q "lychee $(LYCHEE_VERSION)"; then \
		echo "Installing lychee $(LYCHEE_VERSION) into $(LYCHEE_ROOT)..."; \
		mkdir -p "$(LYCHEE_ROOT)"; \
		cargo install --root "$(LYCHEE_ROOT)" lychee --version "$(LYCHEE_VERSION)" --locked; \
	fi

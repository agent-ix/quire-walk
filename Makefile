# =============================================================================
# quire-walk Makefile
# =============================================================================

CARGO ?= cargo
# A target with no `std` at all: building for it proves the crate pulls in
# nothing from `std`.
NO_STD_TARGET ?= thumbv7em-none-eabi

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make build-no-std     - Build for $(NO_STD_TARGET)"
	@echo "  make kani             - cargo kani proofs (TC-899); needs cargo-kani, not part of ci"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check"
	@echo "  make docs             - cargo doc, denying missing/broken doc links"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + build-no-std + deny + docs)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets -- -D warnings
	$(CARGO) clippy --lib --target $(NO_STD_TARGET) -- -D warnings

.PHONY: test
test:
	$(CARGO) test

# `quire-walk` (ADR-011 layer W) is `#![no_std]` with no dependency beyond
# `core` and `alloc`, so it builds for a bare-metal target.
.PHONY: build-no-std
build-no-std:
	$(CARGO) build --locked --target $(NO_STD_TARGET)

# The Kani harnesses in src/proofs.rs (FR-356-AC-4, TC-899).
.PHONY: kani
kani:
	$(CARGO) kani

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

# =============================================================================
# Documentation
# =============================================================================

# cargo doc is a separate lint pass from clippy: rustdoc-only lints such as
# rustdoc::broken_intra_doc_links only fire here, never under `make lint`.
.PHONY: docs
docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --all-features

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test build-no-std deny docs

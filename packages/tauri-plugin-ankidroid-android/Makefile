.PHONY: help format format-check lint lint-fix check build test clean audit outdated machete quickfix ci install-tools

# Default target
help:
	@echo "Available targets:"
	@echo "  format        - Format code with rustfmt"
	@echo "  format-check  - Check formatting without changes"
	@echo "  lint          - Run clippy linter with warnings as errors"
	@echo "  lint-fix      - Auto-fix linting issues"
	@echo "  check         - Type-check all targets"
	@echo "  build         - Build in release mode"
	@echo "  test          - Run all tests"
	@echo "  clean         - Remove build artifacts"
	@echo "  audit         - Check for security vulnerabilities (requires cargo-audit)"
	@echo "  outdated      - Check for outdated dependencies (requires cargo-outdated)"
	@echo "  machete       - Check for unused dependencies (requires cargo-machete)"
	@echo "  quickfix      - Run format and lint-fix"
	@echo "  ci            - Run all checks for CI"
	@echo "  install-tools - Install external cargo tools"

# Formatting
format:
	cargo fmt

format-check:
	cargo fmt -- --check

# Linting
lint:
	cargo clippy -- -D warnings

lint-fix:
	cargo clippy --fix --allow-dirty --allow-staged

# Type checking
check:
	cargo check --all-targets

# Building
build:
	cargo build --release --no-default-features --target-dir target

# Testing
test:
	cargo test

# Cleaning
clean:
	cargo clean

# External tools (require installation)
audit:
	@command -v cargo-audit >/dev/null 2>&1 || (echo "Error: cargo-audit not installed. Run 'make install-tools'" && exit 1)
	cargo audit

outdated:
	@command -v cargo-outdated >/dev/null 2>&1 || (echo "Error: cargo-outdated not installed. Run 'make install-tools'" && exit 1)
	cargo outdated

machete:
	@command -v cargo-machete >/dev/null 2>&1 || (echo "Error: cargo-machete not installed. Run 'make install-tools'" && exit 1)
	cargo machete

# Combined commands
quickfix: format lint-fix

ci: format-check lint check test

# Install external tools
install-tools:
	@echo "Installing external cargo tools..."
	@echo "This may take a few minutes..."
	cargo install cargo-machete cargo-audit cargo-outdated
	@echo "Tools installed successfully!"
# Firefox WebDriver
# Rust 1.92+

.PHONY: help build test check doc format lint clean run-example

help:
	@echo "Build"
	@echo ""
	@echo "  make build       - Build release"
	@echo "  make check       - Check without building"
	@echo "  make doc         - Generate docs"
	@echo ""
	@echo "Test"
	@echo ""
	@echo "  make test        - Run tests"
	@echo "  make run-example - Run example (EXAMPLE=01_basic_launch)"
	@echo ""
	@echo "Quality"
	@echo ""
	@echo "  make format      - Format code"
	@echo "  make lint        - Run clippy"
	@echo ""
	@echo "Clean"
	@echo ""
	@echo "  make clean       - Clean artifacts"

# Build

build:
	@cargo build --release

check:
	@cargo check

doc:
	@cargo doc --no-deps --open

# Test

test:
	@cargo test

run-example:
ifndef EXAMPLE
	@echo "Usage: make run-example EXAMPLE=01_basic_launch"
	@exit 1
endif
	@cargo run --example $(EXAMPLE)

# Quality

format:
	@cargo fmt

lint:
	@cargo clippy -- -D warnings

# Clean

clean:
	@rm -rf target
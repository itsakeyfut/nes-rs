# NES Emulator Justfile
# Convenient shortcuts for common development tasks
#
# Install just: cargo install just
# Usage: just <recipe>

# Default recipe (show help)
default:
    @just --list

# Build targets
# Build the emulator in release mode
build:
    cargo build --release

# Build the emulator in debug mode
build-dev:
    cargo build

# Run the emulator
run:
    cargo run --release

# Run with a specific ROM file
run-rom ROM:
    cargo run --release -- {{ROM}}

# Testing
# Run unit tests (not ignored)
test:
    cargo test

# Run all integration tests
test-all:
    ./tests/run_all_tests.sh

# Run CPU test suite
test-cpu:
    ./tests/run_all_tests.sh --cpu

# Run PPU test suite
test-ppu:
    ./tests/run_all_tests.sh --ppu

# Run APU test suite
test-apu:
    ./tests/run_all_tests.sh --apu

# Run sprite test suite
test-sprite:
    ./tests/run_all_tests.sh --sprite

# Run nestest validation
test-nestest:
    ./tests/run_all_tests.sh --nestest

# Run tests with verbose output
test-verbose:
    ./tests/run_all_tests.sh --verbose

# Generate JSON test report
test-json:
    ./tests/run_all_tests.sh --json

# Run quick smoke test
test-quick:
    cargo test nestest_quick_smoke_test -- --nocapture

# Direct test commands (bypass automation script)
# Run CPU tests directly with cargo
test-cpu-direct:
    cargo test --test blargg_cpu_tests -- --ignored --nocapture

# Run PPU tests directly with cargo
test-ppu-direct:
    cargo test --test blargg_ppu_tests -- --ignored --nocapture

# Run APU tests directly with cargo
test-apu-direct:
    cargo test --test blargg_apu_tests -- --ignored --nocapture

# Run sprite tests directly with cargo
test-sprite-direct:
    cargo test --test sprite_tests -- --ignored --nocapture

# Run a specific test by name
test-one TEST:
    cargo test {{TEST}} -- --ignored --nocapture

# Code Quality
# Format code
fmt:
    cargo fmt

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Run clippy linter
clippy:
    cargo clippy -- -D warnings

# Run clippy with all features
clippy-all:
    cargo clippy --all-features -- -D warnings

# Generate documentation
doc:
    cargo doc --no-deps --open

# Documentation without opening browser
doc-no-open:
    cargo doc --no-deps

# Maintenance
# Clean build artifacts
clean:
    cargo clean
    rm -f test_results.txt test_results.json nestest_trace.log

# Clean and rebuild
rebuild: clean build

# Test ROM Management
# Initialize test ROM submodule
init-roms:
    git submodule update --init --recursive

# Update test ROM submodule
update-roms:
    git submodule update --remote tests/nes-test-rom

# Check if test ROMs are initialized
check-roms:
    #!/usr/bin/env bash
    if [ ! -d "tests/nes-test-rom" ] || [ -z "$(ls -A tests/nes-test-rom)" ]; then
        echo "❌ Test ROM submodule not initialized!"
        echo "Run: just init-roms"
        exit 1
    else
        echo "✅ Test ROMs are initialized"
    fi

# Validation and CI
# Run all checks before committing
validate: fmt clippy test
    @echo ""
    @echo "✅ Code formatted"
    @echo "✅ Clippy checks passed"
    @echo "✅ Unit tests passed"
    @echo ""
    @echo "Ready to commit!"

# Full CI pipeline
ci: validate test-all
    @echo ""
    @echo "✅ All checks passed"
    @echo "✅ All tests passed"
    @echo ""
    @echo "Ready for production!"

# Quick pre-commit check
pre-commit: fmt-check clippy test-quick
    @echo "✅ Pre-commit checks passed"

# Development workflow
# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch and run specific test
watch-test TEST:
    cargo watch -x "test {{TEST}} -- --nocapture"

# Benchmarking
# Run benchmarks (if available)
bench:
    cargo bench

# Performance profiling with perf
profile ROM:
    cargo build --release
    perf record --call-graph=dwarf ./target/release/nes-rs {{ROM}}
    perf report

# Release
# Build optimized release binary
release:
    cargo build --release --locked

# Install the emulator locally
install:
    cargo install --path .

# Create a distributable package
package: release
    #!/usr/bin/env bash
    VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
    mkdir -p dist
    cp target/release/nes-rs dist/
    cp README.md dist/
    tar -czf dist/nes-rs-${VERSION}-linux-x64.tar.gz -C dist nes-rs README.md
    echo "✅ Package created: dist/nes-rs-${VERSION}-linux-x64.tar.gz"

# Development utilities
# Count lines of code
loc:
    tokei

# Find TODO comments
todos:
    rg "TODO|FIXME|XXX|HACK" --type rust

# Show dependency tree
deps:
    cargo tree

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Security audit
audit:
    cargo audit

# Git helpers
# Create a new branch for development
branch NAME:
    git checkout -b {{NAME}}

# Show current git status
status:
    git status

# Quick commit and push
commit MESSAGE: validate
    git add .
    git commit -m "{{MESSAGE}}"
    git push

# Create and view test report
report: test-json
    cat test_results.json | jq '.'

# Debug helpers
# Run with debug output
debug ROM:
    RUST_LOG=debug cargo run --release -- {{ROM}}

# Run with trace output
trace ROM:
    RUST_LOG=trace cargo run --release -- {{ROM}}

# Check binary size
size:
    cargo build --release
    ls -lh target/release/nes-rs
    @echo ""
    @echo "Detailed size breakdown:"
    cargo bloat --release

# Development environment setup
# Setup development environment
setup:
    @echo "Setting up development environment..."
    cargo install just
    cargo install cargo-watch
    cargo install tokei
    cargo install cargo-outdated
    cargo install cargo-audit
    cargo install cargo-bloat
    just init-roms
    @echo "✅ Development environment ready!"

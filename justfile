# Default recipe
default: all

# Go parameters
gocmd := "go"
binary_name := "hail-mary"
binary_path := "./bin/" + binary_name
build_dir := "./bin"
main_path := "./main.go"

# Rust/WASM parameters
wasm_dir := "rust-wasm"
wasm_target := "wasm32-unknown-unknown"
wasm_output := "internal/wasm/module.wasm"

# Version information
version := `git describe --tags --always --dirty 2>/dev/null || echo "dev"`
build_time := `date -u '+%Y-%m-%d_%H:%M:%S'`
ldflags := "-ldflags \"-X main.Version=" + version + " -X main.BuildTime=" + build_time + "\""

# Show help
help:
    @just --list

# Run all checks
all: fmt lint parallel-checks

# Run test and build in parallel
parallel-checks: wasm-build
    @echo "Running tests and build in parallel..."
    @just test & just build & wait
    @echo "All checks completed"

# Install Rust WASM target
install-rust-target:
    @if ! rustup target list --installed | grep -q {{wasm_target}}; then \
        echo "Installing Rust WASM target..."; \
        rustup target add {{wasm_target}}; \
    else \
        echo "Rust WASM target already installed"; \
    fi

# Install tools
install-tools: install-rust-target
    @echo "Installing Go tools..."
    go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
    go install golang.org/x/tools/cmd/goimports@latest
    go install honnef.co/go/tools/cmd/staticcheck@latest
    @echo "Installing Rust tools..."
    @if ! command -v wasm-opt >/dev/null 2>&1; then \
        echo "Installing wasm-opt..."; \
        cargo install wasm-opt --locked; \
    else \
        echo "wasm-opt already installed"; \
    fi

# Build WASM module
wasm-build: install-rust-target
    @echo "Building WASM module..."
    cd {{wasm_dir}} && cargo build --target {{wasm_target}} --release
    @mkdir -p $(dirname {{wasm_output}})
    cp {{wasm_dir}}/target/{{wasm_target}}/release/hail_mary_wasm.wasm {{wasm_output}}
    @if command -v wasm-opt >/dev/null 2>&1; then \
        echo "Optimizing WASM module..."; \
        wasm-opt -Oz {{wasm_output}} -o {{wasm_output}}; \
    else \
        echo "wasm-opt not found, skipping optimization"; \
    fi

# Build Go binary (with WASM)
build: wasm-build
    @echo "Building {{binary_name}}..."
    @mkdir -p {{build_dir}}
    {{gocmd}} build {{ldflags}} -o {{binary_path}} -v {{main_path}}
    @echo "Build complete: {{binary_path}}"

# Build Go binary only (no WASM rebuild)
build-go:
    @echo "Building {{binary_name}} (Go only)..."
    @mkdir -p {{build_dir}}
    {{gocmd}} build {{ldflags}} -o {{binary_path}} -v {{main_path}}

# Clean build artifacts
clean:
    @echo "Cleaning..."
    {{gocmd}} clean
    rm -rf {{build_dir}}
    cd {{wasm_dir}} && cargo clean
    rm -f {{wasm_output}}
    @echo "Clean complete"

# Run tests
test:
    @echo "Running Go tests..."
    {{gocmd}} test -v -race -timeout 30s -cover ./...
    @echo "Running Rust tests..."
    cd {{wasm_dir}} && cargo test

# Run tests with coverage
coverage:
    @echo "Running tests with coverage..."
    {{gocmd}} test -v -race -coverprofile=coverage.out ./...
    {{gocmd}} tool cover -html=coverage.out -o coverage.html
    @echo "Coverage report generated: coverage.html"

# Run linters
lint: wasm-lint
    @echo "Running goimports (auto-fix)..."
    @if command -v goimports >/dev/null 2>&1; then \
        goimports -w .; \
    else \
        echo "goimports not installed. Install with: go install golang.org/x/tools/cmd/goimports@latest"; \
        exit 1; \
    fi
    @echo "Running staticcheck (no auto-fix available)..."
    @if command -v staticcheck >/dev/null 2>&1; then \
        staticcheck ./...; \
    else \
        echo "staticcheck not installed. Install with: go install honnef.co/go/tools/cmd/staticcheck@latest"; \
        exit 1; \
    fi
    @echo "Running golangci-lint (with auto-fix)..."
    @if command -v golangci-lint >/dev/null 2>&1; then \
        golangci-lint run --fix ./...; \
    else \
        echo "golangci-lint not installed. Install with: go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest"; \
        exit 1; \
    fi

# Lint Rust code
wasm-lint:
    @echo "Running Rust linter..."
    cd {{wasm_dir}} && cargo clippy -- -D warnings

# Format code
fmt: wasm-fmt
    @echo "Formatting Go code..."
    gofmt -s -w .
    {{gocmd}} fmt ./...

# Format Rust code
wasm-fmt:
    @echo "Formatting Rust code..."
    cd {{wasm_dir}} && cargo fmt

# Run the application
run: build
    @echo "Running {{binary_name}}..."
    {{binary_path}}

# Run WASM example
run-wasm-example: wasm-build
    @echo "Running WASM example..."
    {{gocmd}} run examples/wasm-hello/main.go

# Install the binary
install: build
    @echo "Installing {{binary_name}}..."
    @cp {{binary_path}} $(go env GOPATH)/bin/{{binary_name}}
    @echo "Installed to $(go env GOPATH)/bin/{{binary_name}}"

# Development mode with live reload
dev:
    @echo "Running in development mode..."
    @if command -v air >/dev/null 2>&1; then \
        air; \
    else \
        echo "Air not installed. Install with: go install github.com/cosmtrek/air@latest"; \
        echo "Running without live reload..."; \
        {{gocmd}} run {{main_path}}; \
    fi

# Watch for changes
watch:
    @echo "Watching for file changes..."
    @if command -v air >/dev/null 2>&1; then \
        echo "Using Air for file watching and rebuilding..."; \
        air --build.cmd "just build" --build.bin "" --build.exclude_dir "bin,vendor,tmp,rust-wasm/target" --build.include_ext "go,rs"; \
    elif command -v watchexec >/dev/null 2>&1; then \
        echo "Using watchexec for file watching..."; \
        watchexec -e go,rs -r "just build"; \
    elif command -v fswatch >/dev/null 2>&1; then \
        echo "Using fswatch for file watching..."; \
        fswatch -o . -e ".*" -i "\\.go$$|\\.rs$$" | xargs -n1 -I{} just build; \
    else \
        echo "No file watcher found. Install one of these:"; \
        echo "  - air: go install github.com/cosmtrek/air@latest"; \
        echo "  - watchexec: brew install watchexec"; \
        echo "  - fswatch: brew install fswatch"; \
        exit 1; \
    fi

# Install dependencies
deps:
    @echo "Downloading Go dependencies..."
    {{gocmd}} get -v -d ./...
    @echo "Downloading Rust dependencies..."
    cd {{wasm_dir}} && cargo fetch

# Tidy dependencies
tidy:
    @echo "Tidying Go dependencies..."
    {{gocmd}} mod tidy -v
    @echo "Checking Rust dependencies..."
    cd {{wasm_dir}} && cargo update --dry-run

# Cross-compilation
build-linux: wasm-build
    @echo "Building for Linux..."
    @mkdir -p {{build_dir}}
    GOOS=linux GOARCH=amd64 {{gocmd}} build {{ldflags}} -o {{build_dir}}/{{binary_name}}-linux-amd64 -v {{main_path}}

build-windows: wasm-build
    @echo "Building for Windows..."
    @mkdir -p {{build_dir}}
    GOOS=windows GOARCH=amd64 {{gocmd}} build {{ldflags}} -o {{build_dir}}/{{binary_name}}-windows-amd64.exe -v {{main_path}}

build-mac: wasm-build
    @echo "Building for macOS..."
    @mkdir -p {{build_dir}}
    GOOS=darwin GOARCH=amd64 {{gocmd}} build {{ldflags}} -o {{build_dir}}/{{binary_name}}-darwin-amd64 -v {{main_path}}
    GOOS=darwin GOARCH=arm64 {{gocmd}} build {{ldflags}} -o {{build_dir}}/{{binary_name}}-darwin-arm64 -v {{main_path}}

build-all: build-linux build-windows build-mac

# CI checks
check: fmt lint test
ci: deps tidy check build
# Go parameters
GOCMD=go
GOBUILD=$(GOCMD) build
GOCLEAN=$(GOCMD) clean
GOTEST=$(GOCMD) test
GOGET=$(GOCMD) get
GOMOD=$(GOCMD) mod
GOFMT=gofmt
GOLINT=golangci-lint

# Binary name
BINARY_NAME=hail-mary
BINARY_PATH=./bin/$(BINARY_NAME)

# Build variables
BUILD_DIR=./bin
MAIN_PATH=./main.go

# Version information
VERSION=$(shell git describe --tags --always --dirty)
BUILD_TIME=$(shell date -u '+%Y-%m-%d_%H:%M:%S')
LDFLAGS=-ldflags "-X main.Version=$(VERSION) -X main.BuildTime=$(BUILD_TIME)"

.PHONY: all build clean test coverage lint fmt help run install deps tidy parallel-checks

## help: Show this help message
help:
	@echo 'Usage:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

## all: Run fmt, lint, test, and build (test and build run in parallel)
all: fmt lint parallel-checks

## parallel-checks: Run test and build in parallel
parallel-checks:
	@$(MAKE) --no-print-directory -j2 test build
	@echo "All checks completed"

## build: Build the binary
build:
	@echo "Building $(BINARY_NAME)..."
	@mkdir -p $(BUILD_DIR)
	$(GOBUILD) $(LDFLAGS) -o $(BINARY_PATH) -v $(MAIN_PATH)
	@echo "Build complete: $(BINARY_PATH)"

## clean: Remove build artifacts
clean:
	@echo "Cleaning..."
	$(GOCLEAN)
	@rm -rf $(BUILD_DIR)
	@echo "Clean complete"

## test: Run tests
test:
	@echo "Running tests..."
	$(GOTEST) -v -race -timeout 30s ./...

## coverage: Run tests with coverage
coverage:
	@echo "Running tests with coverage..."
	$(GOTEST) -v -race -coverprofile=coverage.out ./...
	$(GOCMD) tool cover -html=coverage.out -o coverage.html
	@echo "Coverage report generated: coverage.html"

## lint: Run linter
lint:
	@echo "Running linter..."
	@if command -v golangci-lint >/dev/null 2>&1; then \
		$(GOLINT) run ./...; \
	else \
		echo "golangci-lint not installed. Install with: go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest"; \
		exit 1; \
	fi

## fmt: Format code
fmt:
	@echo "Formatting code..."
	$(GOFMT) -s -w .
	$(GOCMD) fmt ./...

## run: Run the application
run: build
	@echo "Running $(BINARY_NAME)..."
	$(BINARY_PATH)

## install: Install the binary
install: build
	@echo "Installing $(BINARY_NAME)..."
	@cp $(BINARY_PATH) $(GOPATH)/bin/$(BINARY_NAME)
	@echo "Installed to $(GOPATH)/bin/$(BINARY_NAME)"

## deps: Download dependencies
deps:
	@echo "Downloading dependencies..."
	$(GOGET) -v -d ./...

## tidy: Tidy up module dependencies
tidy:
	@echo "Tidying dependencies..."
	$(GOMOD) tidy -v

## dev: Run the application in development mode (with live reload if available)
dev:
	@echo "Running in development mode..."
	@if command -v air >/dev/null 2>&1; then \
		air; \
	else \
		echo "Air not installed. Install with: go install github.com/cosmtrek/air@latest"; \
		echo "Running without live reload..."; \
		$(GOCMD) run $(MAIN_PATH); \
	fi

## list: Run the list command
list: build
	$(BINARY_PATH) list

## ui: Run the UI command
ui: build
	$(BINARY_PATH) ui

# Cross-compilation targets
## build-linux: Build for Linux
build-linux:
	@echo "Building for Linux..."
	@mkdir -p $(BUILD_DIR)
	GOOS=linux GOARCH=amd64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-linux-amd64 -v $(MAIN_PATH)

## build-windows: Build for Windows
build-windows:
	@echo "Building for Windows..."
	@mkdir -p $(BUILD_DIR)
	GOOS=windows GOARCH=amd64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-windows-amd64.exe -v $(MAIN_PATH)

## build-mac: Build for macOS
build-mac:
	@echo "Building for macOS..."
	@mkdir -p $(BUILD_DIR)
	GOOS=darwin GOARCH=amd64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-darwin-amd64 -v $(MAIN_PATH)
	GOOS=darwin GOARCH=arm64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-darwin-arm64 -v $(MAIN_PATH)

## build-all: Build for all platforms
build-all: build-linux build-windows build-mac

## check: Run fmt, lint, and test
check: fmt lint test

## ci: Run continuous integration checks
ci: deps tidy check build

.DEFAULT_GOAL := help
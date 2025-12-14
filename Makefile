.PHONY: help build test clean run-api fmt lint

help: ## Show this help message
	@echo "TGP - The Grid Platform"
	@echo "Usage: make [target]"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

build: ## Build all components (Rust + Go)
	@echo "Building Rust components..."
	cargo build --workspace --release
	@echo "Building Go API server..."
	cd api && go build -o tgp-api .
	@echo "Build complete!"

test: ## Run all tests
	@echo "Running Rust tests..."
	cargo test --workspace
	@echo "Running Go tests..."
	cd api && go test ./...
	@echo "All tests passed!"

clean: ## Clean build artifacts
	cargo clean
	cd api && go clean
	rm -f api/tgp-api

run-api: ## Run API server
	cd api && go run .

fmt: ## Format code
	cargo fmt --all
	cd api && go fmt ./...

lint: ## Run linters
	cargo clippy --workspace -- -D warnings
	cd api && go vet ./...

bench: ## Run benchmarks
	cargo bench --workspace

coverage: ## Generate code coverage report
	cargo tarpaulin --workspace --out Html --output-dir coverage

dev: ## Run in development mode (auto-reload)
	@echo "Starting TGP in development mode..."
	@echo "API server on http://localhost:8080"
	cd api && go run .

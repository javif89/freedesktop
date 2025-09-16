.PHONY: help check test publish-dry-run publish clean

# Default target
help:
	@echo "freedesktop workspace publishing targets:"
	@echo ""
	@echo "  check           - Check all crates compile"
	@echo "  test            - Run all tests"
	@echo "  publish-dry-run - Dry run publish for all crates"
	@echo "  publish         - Publish all crates in correct order"
	@echo "  clean           - Clean build artifacts"
	@echo ""
	@echo "For first-time publishing, run: make publish"

# Development targets
check:
	@echo "🔍 Checking all crates..."
	cargo check --workspace

test:
	@echo "🧪 Running tests..."
	cargo test --workspace

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Dry run - test publishing without actually doing it
publish-dry-run:
	@echo "🔍 Dry run publishing all crates in order..."
	@echo "📦 1/3 Dry run: freedesktop-core"
	cargo publish --dry-run -p freedesktop-core
	@echo "📦 2/3 Dry run: freedesktop-apps"
	cargo publish --dry-run -p freedesktop-apps
	@echo "📦 3/3 Dry run: freedesktop (umbrella)"
	cargo publish --dry-run -p freedesktop
	@echo "✅ All dry runs completed successfully!"

# Publish all crates in correct dependency order
publish: check test
	@echo "🚀 Publishing all crates in dependency order..."
	@echo ""
	@echo "This will publish to crates.io:"
	@echo "  1. freedesktop-core"
	@echo "  2. freedesktop-apps (depends on core)"
	@echo "  3. freedesktop (umbrella, depends on both)"
	@echo ""
	@read -p "Continue? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	@echo ""
	@echo "📦 1/3 Publishing freedesktop-core..."
	cargo publish -p freedesktop-core
	@echo "⏳ Waiting 60 seconds for crates.io to index freedesktop-core..."
	sleep 60
	@echo "📦 2/3 Publishing freedesktop-apps..."
	cargo publish -p freedesktop-apps
	@echo "⏳ Waiting 60 seconds for crates.io to index freedesktop-apps..."
	sleep 60
	@echo "📦 3/3 Publishing freedesktop (umbrella)..."
	cargo publish -p freedesktop
	@echo ""
	@echo "🎉 All crates published successfully!"
	@echo "📋 Next steps:"
	@echo "  • Check https://crates.io/crates/freedesktop"
	@echo "  • Check https://crates.io/crates/freedesktop-core"
	@echo "  • Check https://crates.io/crates/freedesktop-apps"
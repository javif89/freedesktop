#!/bin/bash
set -e

echo "🚀 Publishing freedesktop workspace crates..."

# Function to restore dependencies on exit
cleanup() {
    echo "🔄 Restoring development dependencies..."
    
    # Restore freedesktop-apps dependency
    sed -i 's/freedesktop-core = "0.1.0"/freedesktop-core = { version = "0.1.0", path = "..\/freedesktop-core" }/' freedesktop-apps/Cargo.toml
    
    # Restore umbrella crate dependencies  
    sed -i 's/freedesktop-core = { version = "0.1.0", optional = true }/freedesktop-core = { version = "0.1.0", path = "..\/freedesktop-core", optional = true }/' freedesktop/Cargo.toml
    sed -i 's/freedesktop-apps = { version = "0.1.0", optional = true }/freedesktop-apps = { version = "0.1.0", path = "..\/freedesktop-apps", optional = true }/' freedesktop/Cargo.toml
    
    echo "✅ Dependencies restored for development"
}

# Set up cleanup on exit
trap cleanup EXIT

# Run tests first
echo "🧪 Running tests..."
cargo test --workspace

# Publish freedesktop-core
echo "📦 Publishing freedesktop-core..."
cd freedesktop-core
cargo publish
cd ..

echo "⏳ Waiting 30 seconds for freedesktop-core to be available on crates.io..."
sleep 30

# Publish freedesktop-apps  
echo "📦 Publishing freedesktop-apps..."
cd freedesktop-apps
cargo publish
cd ..

echo "⏳ Waiting 30 seconds for freedesktop-apps to be available on crates.io..."
sleep 30

# Publish umbrella crate
echo "📦 Publishing freedesktop (umbrella)..."
cd freedesktop
cargo publish
cd ..

echo "🎉 All crates published successfully!"
echo "📋 Next steps:"
echo "  • Check https://crates.io/crates/freedesktop"
echo "  • Check https://crates.io/crates/freedesktop-core"
echo "  • Check https://crates.io/crates/freedesktop-apps"
echo "  • Update version numbers for next release"
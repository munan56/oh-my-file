#!/bin/bash

# Clean script for oh-my-file
# Removes build artifacts and caches

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🧹 Cleaning build artifacts..."
echo ""

cd "$PROJECT_ROOT"

# Function to show directory size
show_size() {
  if [ -d "$1" ]; then
    size=$(du -sh "$1" 2>/dev/null | cut -f1)
    echo "✓ Removed: $1 ($size)"
  fi
}

# Clean directories
[ -d "dist" ] && rm -rf dist && show_size "dist" || true
[ -d "src-tauri/target" ] && rm -rf src-tauri/target && show_size "src-tauri/target" || true

echo ""
echo "✅ Cleanup complete!"

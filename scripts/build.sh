#!/bin/bash

# Build script for oh-my-file
# Usage: ./scripts/build.sh [TARGET]

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-all}"

echo "🔨 Building oh-my-file"
echo "📁 Project root: $PROJECT_ROOT"
echo "🎯 Target: $TARGET"

cd "$PROJECT_ROOT"

case "$TARGET" in
  all|current)
    echo "📝 Building for current platform..."
    npm run package
    ;;
  win32|windows)
    echo "📝 Building for Windows..."
    npm run package win32
    ;;
  darwin|macos)
    echo "📝 Building for macOS..."
    npm run package darwin
    ;;
  linux)
    echo "📝 Building for Linux..."
    npm run package linux
    ;;
  *)
    echo "❌ Unknown target: $TARGET"
    echo "Available targets: all, win32, darwin, linux"
    exit 1
    ;;
esac

echo "✅ Build complete!"

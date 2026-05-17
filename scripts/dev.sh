#!/bin/bash

# Development server script for oh-my-file
# Starts the Tauri dev server with hot reload

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🚀 Starting development server..."
echo "📁 Project root: $PROJECT_ROOT"
echo ""

cd "$PROJECT_ROOT"

npm run tauri dev

echo ""
echo "✅ Dev server stopped"

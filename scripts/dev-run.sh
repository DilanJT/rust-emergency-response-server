#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | sed 's/#.*//g' | xargs)
fi

echo "🚀 Starting Dubai Healthcare Emergency Response System"
echo "📍 Server will be available at: http://localhost:${SERVER_PORT:-3000}"

# Install cargo-watch if not present
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch for hot reloading..."
    cargo install cargo-watch
fi

# Run with hot reloading
cargo watch -q -c -w crates/ -x "run -p web-server"

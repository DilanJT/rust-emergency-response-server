#!/bin/bash
set -e

echo "🔄 Running database migrations..."

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | sed 's/#.*//g' | xargs)
fi

# Run migration service
cargo run -p migration

echo "✅ Migrations completed!"

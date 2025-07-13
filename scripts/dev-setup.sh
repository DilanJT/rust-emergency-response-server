#!/bin/bash
set -e

echo "🏥 Setting up Dubai Healthcare Emergency Response System"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Start database services
echo "🐘 Starting PostgreSQL and Redis..."
docker-compose up -d

# Wait for postgres to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until docker exec healthcare_postgres pg_isready -U admin -d healthcare_emergency; do
    sleep 1
done

echo "📦 Installing dependencies..."
cargo check

echo "🔧 Running database migrations..."
# TODO: Run migrations when implemented
# cargo run -p migration

echo "🌱 Seeding development data..."
# TODO: Run seed data when implemented  
# cargo run -p seed-data

echo "✅ Setup complete! You can now run:"
echo "   cargo run -p web-server"

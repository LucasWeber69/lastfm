#!/bin/bash

# Last.fm Dating App - Stop Script
# This script stops all application services

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

echo "🛑 Stopping Last.fm Dating App Services..."
echo ""

# Stop backend (Rust server on port 8000)
print_info "Stopping backend..."
pkill -f "cargo run" || print_info "Backend not running"
lsof -ti:8000 | xargs kill -9 2>/dev/null || true
print_success "Backend stopped"

# Stop frontend (Vite dev server on port 3000)
print_info "Stopping frontend..."
pkill -f "vite" || print_info "Frontend not running"
lsof -ti:3000 | xargs kill -9 2>/dev/null || true
print_success "Frontend stopped"

# Optionally stop Redis and MinIO (commented out by default as they might be used by other apps)
# print_info "Stopping Redis..."
# brew services stop redis 2>/dev/null || redis-cli shutdown 2>/dev/null || true

# print_info "Stopping MinIO..."
# pkill -f "minio server" || true

echo ""
print_success "Application services stopped"
echo ""
echo "Note: Redis and MinIO are still running (they may be used by other applications)"
echo "To stop them manually:"
echo "  Redis: brew services stop redis"
echo "  MinIO: pkill -f 'minio server'"
echo ""

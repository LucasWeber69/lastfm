#!/bin/bash

# Last.fm Dating App - Start Script
# This script starts all required services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

print_header() {
    echo -e "${BLUE}$1${NC}"
}

# Function to check if a service is running
check_service() {
    if pgrep -x "$1" > /dev/null; then
        return 0
    else
        return 1
    fi
}

# Function to check if a port is in use
check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

clear
print_header "🎵 Last.fm Dating App - Starting Services"
print_header "=========================================="
echo ""

# Change to project root
cd "$(dirname "$0")/.."

# Start Redis
print_info "Starting Redis..."
if check_service "redis-server"; then
    print_success "Redis is already running"
elif command -v redis-server >/dev/null 2>&1; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew services start redis
    else
        redis-server --daemonize yes
    fi
    sleep 1
    print_success "Redis started"
else
    print_error "Redis not found. Please install it first."
    exit 1
fi

# Start MinIO
print_info "Starting MinIO..."
if check_port 9000; then
    print_success "MinIO is already running"
elif command -v minio >/dev/null 2>&1; then
    if [ -f "$HOME/.minio/start-minio.sh" ]; then
        nohup "$HOME/.minio/start-minio.sh" > "$HOME/.minio/minio.log" 2>&1 &
    else
        mkdir -p "$HOME/.minio/data"
        export MINIO_ROOT_USER=minioadmin
        export MINIO_ROOT_PASSWORD=minioadmin
        nohup minio server "$HOME/.minio/data" --console-address ":9001" > "$HOME/.minio/minio.log" 2>&1 &
    fi
    sleep 2
    print_success "MinIO started"
else
    print_error "MinIO not found. Please install it first."
    exit 1
fi

# Check MySQL
print_info "Checking MySQL..."
if command -v mysql >/dev/null 2>&1; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew services start mysql 2>/dev/null || true
    fi
    print_success "MySQL is ready"
else
    print_error "MySQL not found. Please install it first."
    exit 1
fi

echo ""
print_header "Starting Application Services"
print_header "==============================="
echo ""

# Create log directory
mkdir -p logs

# Start Backend
print_info "Starting backend server..."
if check_port 8000; then
    print_info "Backend is already running on port 8000"
else
    cd backend
    nohup cargo run --release > ../logs/backend.log 2>&1 &
    BACKEND_PID=$!
    cd ..
    print_success "Backend starting (PID: $BACKEND_PID)"
fi

# Start Frontend
print_info "Starting frontend development server..."
if check_port 3000; then
    print_info "Frontend is already running on port 3000"
else
    cd frontend
    nohup npm run dev > ../logs/frontend.log 2>&1 &
    FRONTEND_PID=$!
    cd ..
    print_success "Frontend starting (PID: $FRONTEND_PID)"
fi

# Wait for services to start
print_info "Waiting for services to start..."
sleep 5

echo ""
print_header "=========================================="
print_success "All services started!"
print_header "=========================================="
echo ""
echo "Access the application:"
echo -e "  ${GREEN}Frontend:${NC}      http://localhost:3000"
echo -e "  ${GREEN}Backend API:${NC}   http://localhost:8000"
echo -e "  ${GREEN}MinIO Console:${NC} http://localhost:9001"
echo ""
echo "MinIO Credentials:"
echo "  Username: minioadmin"
echo "  Password: minioadmin"
echo ""
echo "Logs:"
echo "  Backend:  tail -f logs/backend.log"
echo "  Frontend: tail -f logs/frontend.log"
echo "  MinIO:    tail -f $HOME/.minio/minio.log"
echo ""
echo "To stop all services:"
echo "  ./scripts/stop.sh"
echo ""

#!/bin/bash

# Last.fm Dating App - Setup Script for Mac
# This script sets up the development environment on macOS

set -e

echo "🎵 Last.fm Dating App - Setup Script"
echo "===================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to print colored messages
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# Check for macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script is designed for macOS. For other systems, install dependencies manually."
    exit 1
fi

# Check for Homebrew
echo "Checking dependencies..."
if ! command_exists brew; then
    print_error "Homebrew is not installed. Please install it from https://brew.sh/"
    exit 1
fi
print_success "Homebrew is installed"

# Check for Rust
if ! command_exists cargo; then
    print_info "Rust is not installed. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    print_success "Rust installed"
else
    print_success "Rust is installed ($(rustc --version))"
fi

# Check for Node.js
if ! command_exists node; then
    print_info "Node.js is not installed. Installing via Homebrew..."
    brew install node
    print_success "Node.js installed"
else
    print_success "Node.js is installed ($(node --version))"
fi

# Check for MySQL
if ! command_exists mysql; then
    print_info "MySQL is not installed. Installing via Homebrew..."
    brew install mysql
    brew services start mysql
    print_success "MySQL installed and started"
else
    print_success "MySQL is installed"
    # Try to start MySQL if it's not running
    brew services start mysql 2>/dev/null || true
fi

# Check for Redis
if ! command_exists redis-cli; then
    print_info "Redis is not installed. Installing via Homebrew..."
    brew install redis
    brew services start redis
    print_success "Redis installed and started"
else
    print_success "Redis is installed"
    # Try to start Redis if it's not running
    brew services start redis 2>/dev/null || true
fi

# Check for MinIO
if ! command_exists minio; then
    print_info "MinIO is not installed. Installing via Homebrew..."
    brew install minio/stable/minio
    print_success "MinIO installed"
else
    print_success "MinIO is installed"
fi

# Setup MinIO
echo ""
echo "Setting up MinIO..."
MINIO_DATA_DIR="$HOME/.minio/data"
mkdir -p "$MINIO_DATA_DIR"

# Create MinIO start script
cat > "$HOME/.minio/start-minio.sh" << 'EOF'
#!/bin/bash
export MINIO_ROOT_USER=minioadmin
export MINIO_ROOT_PASSWORD=minioadmin
minio server ~/.minio/data --console-address ":9001"
EOF

chmod +x "$HOME/.minio/start-minio.sh"

# Start MinIO in background (if not already running)
if ! pgrep -x "minio" > /dev/null; then
    print_info "Starting MinIO..."
    nohup "$HOME/.minio/start-minio.sh" > "$HOME/.minio/minio.log" 2>&1 &
    sleep 3
    print_success "MinIO started"
else
    print_info "MinIO is already running"
fi

# Create MinIO bucket
echo "Creating MinIO bucket..."
if command_exists mc; then
    mc alias set local http://localhost:9000 minioadmin minioadmin 2>/dev/null || true
    mc mb local/lastfm-photos 2>/dev/null || print_info "Bucket already exists"
    print_success "MinIO bucket 'lastfm-photos' ready"
else
    print_info "MinIO Client (mc) not found. Install it with: brew install minio/stable/mc"
    print_info "Then manually create the bucket: mc mb local/lastfm-photos"
fi

# Setup MySQL database
echo ""
echo "Setting up MySQL database..."
print_info "Creating database 'lastfm_dating'..."

# Check if database exists
if mysql -u root -e "USE lastfm_dating" 2>/dev/null; then
    print_info "Database 'lastfm_dating' already exists"
else
    mysql -u root -e "CREATE DATABASE lastfm_dating CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;" || {
        print_error "Failed to create database. You may need to set a root password."
        print_info "Try: mysql_secure_installation"
        exit 1
    }
    print_success "Database created"
fi

# Run migrations
print_info "Running database migrations..."
cd "$(dirname "$0")/.."
for migration in backend/migrations/*.sql; do
    if [ -f "$migration" ]; then
        print_info "Running $(basename "$migration")..."
        mysql -u root lastfm_dating < "$migration" || {
            print_error "Migration $(basename "$migration") failed. It may have already been run."
        }
    fi
done
print_success "Migrations completed"

# Setup environment files
echo ""
echo "Setting up environment files..."

# Backend .env
if [ ! -f backend/.env ]; then
    cp backend/.env.example backend/.env
    print_success "Created backend/.env"
    print_info "Please edit backend/.env with your configuration (Last.fm API keys, etc.)"
else
    print_info "backend/.env already exists"
fi

# Frontend .env (if needed)
if [ -f frontend/.env.example ] && [ ! -f frontend/.env ]; then
    cp frontend/.env.example frontend/.env
    print_success "Created frontend/.env"
fi

# Install dependencies
echo ""
echo "Installing dependencies..."

# Backend dependencies
print_info "Installing Rust dependencies..."
cd backend
cargo build --release
print_success "Rust dependencies installed"

# Frontend dependencies
print_info "Installing Node.js dependencies..."
cd ../frontend
npm install
print_success "Node.js dependencies installed"

cd ..

# Final instructions
echo ""
echo "======================================"
print_success "Setup complete!"
echo "======================================"
echo ""
echo "Next steps:"
echo "1. Edit backend/.env with your Last.fm API credentials"
echo "2. Start the development servers:"
echo "   ./scripts/start.sh"
echo ""
echo "Or start services individually:"
echo "   Backend:  cd backend && cargo run"
echo "   Frontend: cd frontend && npm run dev"
echo ""
echo "URLs:"
echo "   Frontend: http://localhost:3000"
echo "   Backend:  http://localhost:8000"
echo "   MinIO:    http://localhost:9001 (console)"
echo ""
echo "MinIO credentials:"
echo "   Username: minioadmin"
echo "   Password: minioadmin"
echo ""

#!/bin/bash

# SniperBot 2.0 - Development Mode Startup Script
# This script starts the API server and frontend in development mode

set -e

echo "🚀 Starting SniperBot 2.0 in development mode..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the SniperBot root directory"
    exit 1
fi

# Function to cleanup background processes
cleanup() {
    print_status "Shutting down services..."
    if [ ! -z "$API_PID" ]; then
        kill $API_PID 2>/dev/null || true
    fi
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null || true
    fi
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Step 1: Check configuration
print_status "Checking configuration..."
if [ ! -f "config.toml" ]; then
    if [ -f "config.example.toml" ]; then
        print_warning "config.toml not found, copying from config.example.toml"
        cp config.example.toml config.toml
        print_warning "Please edit config.toml with your API keys and settings"
    else
        print_error "No configuration file found. Please create config.toml"
        exit 1
    fi
fi

# Step 2: Build the API server
print_status "Building SniperBot API server..."
if cargo build -p sniperbot_ui_api --bin api_server; then
    print_success "API server built successfully"
else
    print_error "Failed to build API server"
    exit 1
fi

# Step 3: Start the API server in background
print_status "Starting API server on port 8084..."
cargo run -p sniperbot_ui_api --bin api_server &
API_PID=$!

# Wait a moment for the API server to start
sleep 3

# Step 4: Install frontend dependencies if needed
print_status "Checking frontend dependencies..."
cd frontend
if [ ! -d "node_modules" ]; then
    print_status "Installing frontend dependencies..."
    npm install
fi

# Step 5: Start the frontend development server
print_status "Starting frontend development server on port 3000..."
print_status "Frontend will proxy API requests to http://localhost:8084"
npm run dev &
FRONTEND_PID=$!

cd ..

echo ""
print_success "🎉 SniperBot 2.0 development environment is running!"
echo ""
print_status "Frontend (dev): http://localhost:3000"
print_status "API server: http://localhost:8084"
print_status "WebSocket: ws://localhost:8084/ws"
echo ""
print_status "Press Ctrl+C to stop all services"
echo ""

# Wait for background processes
wait

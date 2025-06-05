#!/bin/bash

# SniperBot 2.0 - Complete System Startup Script
# This script builds the frontend and starts both the bot and API server

set -e

echo "🚀 Starting SniperBot 2.0 with integrated frontend..."

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

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    print_error "Node.js is not installed. Please install Node.js to build the frontend."
    exit 1
fi

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    print_error "npm is not installed. Please install npm to build the frontend."
    exit 1
fi

# Step 1: Install frontend dependencies
print_status "Installing frontend dependencies..."
cd frontend
if npm install; then
    print_success "Frontend dependencies installed"
else
    print_error "Failed to install frontend dependencies"
    exit 1
fi

# Step 2: Build the frontend
print_status "Building frontend for production..."
if npm run build; then
    print_success "Frontend built successfully"
else
    print_error "Failed to build frontend"
    exit 1
fi

# Go back to root directory
cd ..

# Step 3: Build the Rust project
print_status "Building SniperBot Rust components..."
if cargo build --release; then
    print_success "Rust components built successfully"
else
    print_error "Failed to build Rust components"
    exit 1
fi

# Step 4: Check configuration
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

# Step 5: Start the API server (which now serves the frontend)
print_status "Starting SniperBot API server with integrated frontend..."
print_status "Frontend will be available at: http://localhost:8084"
print_status "API endpoints available at: http://localhost:8084/api/*"
print_status "WebSocket available at: ws://localhost:8084/ws"

echo ""
print_success "🎉 SniperBot 2.0 is starting up!"
echo ""
print_status "Press Ctrl+C to stop the server"
echo ""

# Start the API server
cargo run -p sniperbot_ui_api --bin api_server --release

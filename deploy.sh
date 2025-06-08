#!/bin/bash

# SniperBot 2.0 - Production Deployment Script
# 🚀 Deploy to Cloud with Docker

set -e  # Exit on any error

echo "🚀 SniperBot 2.0 - Production Deployment"
echo "========================================"

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

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root for security reasons"
   exit 1
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Check if .env.production exists
if [ ! -f ".env.production" ]; then
    print_error ".env.production file not found!"
    print_warning "Please create .env.production with your production configuration"
    exit 1
fi

# Validate critical environment variables
print_status "Validating production configuration..."

# Check if wallet private key is set
if grep -q "YOUR_PRODUCTION_WALLET_PRIVATE_KEY_HERE" .env.production; then
    print_error "Please set your WALLET_PRIVATE_KEY in .env.production"
    exit 1
fi

# Check if Mistral API key is set (optional but recommended)
if grep -q "YOUR_MISTRAL_API_KEY_HERE" .env.production; then
    print_warning "Mistral API key not set - AI features will use simulation mode"
fi

print_success "Configuration validation passed"

# Create necessary directories
print_status "Creating necessary directories..."
mkdir -p data logs configs prometheus grafana nginx

# Stop existing containers if running
print_status "Stopping existing containers..."
docker-compose -f docker-compose.production.yml down || true

# Pull latest images
print_status "Pulling latest Docker images..."
docker-compose -f docker-compose.production.yml pull

# Build the application
print_status "Building SniperBot application..."
docker-compose -f docker-compose.production.yml build sniper-bot

# Start the services
print_status "Starting production services..."
docker-compose -f docker-compose.production.yml up -d

# Wait for services to be ready
print_status "Waiting for services to start..."
sleep 30

# Check service health
print_status "Checking service health..."

# Check if SniperBot is running
if docker-compose -f docker-compose.production.yml ps sniper-bot | grep -q "Up"; then
    print_success "SniperBot is running"
else
    print_error "SniperBot failed to start"
    docker-compose -f docker-compose.production.yml logs sniper-bot
    exit 1
fi

# Check if Redis is running
if docker-compose -f docker-compose.production.yml ps redis | grep -q "Up"; then
    print_success "Redis is running"
else
    print_warning "Redis may not be running properly"
fi

# Test API endpoint
print_status "Testing API endpoint..."
if curl -f http://localhost:8084/health > /dev/null 2>&1; then
    print_success "API endpoint is responding"
else
    print_warning "API endpoint not responding yet (may need more time)"
fi

# Display service URLs
echo ""
print_success "🎉 SniperBot 2.0 deployed successfully!"
echo ""
echo "📊 Service URLs:"
echo "  • SniperBot API:    http://localhost:8084"
echo "  • Dashboard:        http://localhost:8084"
echo "  • Grafana:          http://localhost:3000 (admin/sniperbot123)"
echo "  • QuestDB Console:  http://localhost:9000"
echo "  • Prometheus:       http://localhost:9090"
echo "  • Neo4j Browser:    http://localhost:7474 (neo4j/sniperbot123)"
echo ""
echo "📋 Management Commands:"
echo "  • View logs:        docker-compose -f docker-compose.production.yml logs -f sniper-bot"
echo "  • Stop services:    docker-compose -f docker-compose.production.yml down"
echo "  • Restart bot:      docker-compose -f docker-compose.production.yml restart sniper-bot"
echo "  • View status:      docker-compose -f docker-compose.production.yml ps"
echo ""
echo "🚨 IMPORTANT:"
echo "  • Bot is in LIVE TRADING mode with real money!"
echo "  • Monitor the dashboard and logs regularly"
echo "  • Set up proper monitoring and alerts"
echo "  • Keep your private keys secure"
echo ""

# Show running containers
print_status "Running containers:"
docker-compose -f docker-compose.production.yml ps

print_success "Deployment complete! 🚀"

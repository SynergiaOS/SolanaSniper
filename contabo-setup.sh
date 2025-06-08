#!/bin/bash

# SniperBot 2.0 - Contabo VDS Setup Script
# 🔧 Setup environment on Contabo server

set -e

echo "🔧 SniperBot 2.0 - Contabo VDS Setup"
echo "===================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Update system
print_status "Updating system packages..."
apt-get update -y
apt-get upgrade -y

# Install Docker if not present
if ! command -v docker &> /dev/null; then
    print_status "Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    systemctl enable docker
    systemctl start docker
    rm get-docker.sh
    print_success "Docker installed"
else
    print_success "Docker already installed"
fi

# Install Docker Compose if not present
if ! command -v docker-compose &> /dev/null; then
    print_status "Installing Docker Compose..."
    curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
    print_success "Docker Compose installed"
else
    print_success "Docker Compose already installed"
fi

# Install additional tools
print_status "Installing additional tools..."
apt-get install -y curl jq htop nano ufw fail2ban

# Configure firewall
print_status "Configuring firewall..."
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 8084/tcp  # SniperBot API
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw --force enable
print_success "Firewall configured"

# Configure fail2ban
print_status "Configuring fail2ban..."
systemctl enable fail2ban
systemctl start fail2ban
print_success "Fail2ban configured"

# Create directories
print_status "Creating directories..."
mkdir -p data logs configs
chown -R 1000:1000 data logs configs

# Stop any existing containers
print_status "Stopping existing containers..."
docker-compose -f docker-compose.production.yml down 2>/dev/null || true

# Build and start SniperBot
print_status "Building SniperBot..."
docker-compose -f docker-compose.production.yml build sniper-bot

print_status "Starting SniperBot..."
docker-compose -f docker-compose.production.yml up -d sniper-bot

# Wait for service to start
print_status "Waiting for SniperBot to start..."
sleep 30

# Check if service is running
if docker-compose -f docker-compose.production.yml ps sniper-bot | grep -q "Up"; then
    print_success "SniperBot is running!"
else
    print_error "SniperBot failed to start"
    print_status "Checking logs..."
    docker-compose -f docker-compose.production.yml logs sniper-bot
    exit 1
fi

# Test API endpoint
print_status "Testing API endpoint..."
if curl -f http://localhost:8084/health > /dev/null 2>&1; then
    print_success "API endpoint is responding"
else
    print_warning "API endpoint not responding yet (may need more time)"
fi

# Display status
print_status "Service status:"
docker-compose -f docker-compose.production.yml ps

# Display system info
print_status "System information:"
echo "  • CPU: $(nproc) cores"
echo "  • RAM: $(free -h | awk '/^Mem:/ {print $2}') total"
echo "  • Disk: $(df -h / | awk 'NR==2 {print $4}') available"
echo "  • Docker: $(docker --version)"

print_success "🎉 SniperBot 2.0 setup completed on Contabo VDS!"
echo ""
echo "📊 Service URLs:"
echo "  • SniperBot API:    http://$(curl -s ifconfig.me):8084"
echo "  • Dashboard:        http://$(curl -s ifconfig.me):8084"
echo ""
echo "📋 Management Commands:"
echo "  • View logs:        docker-compose -f docker-compose.production.yml logs -f sniper-bot"
echo "  • Stop service:     docker-compose -f docker-compose.production.yml down"
echo "  • Restart service:  docker-compose -f docker-compose.production.yml restart sniper-bot"
echo "  • View status:      docker-compose -f docker-compose.production.yml ps"
echo "  • System monitor:   htop"
echo ""
echo "🚨 IMPORTANT:"
echo "  • Bot is in LIVE TRADING mode with real money!"
echo "  • Monitor the logs regularly: docker logs -f sniperbot-production"
echo "  • Keep your private keys secure"
echo "  • Set up monitoring and alerts"
echo ""
print_success "Setup complete! 🚀"

#!/bin/bash

# SniperBot 2.0 - Contabo VDS Deployment Script
# 🚀 Deploy to Contabo Cloud Server

set -e  # Exit on any error

echo "🚀 SniperBot 2.0 - Contabo VDS Deployment"
echo "========================================="

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

# Server configuration
SERVER_IP=${1:-"YOUR_CONTABO_IP"}
SERVER_USER=${2:-"root"}
DEPLOY_PATH="/opt/sniperbot"

if [ "$SERVER_IP" = "YOUR_CONTABO_IP" ]; then
    print_error "Please provide server IP: ./contabo-deploy.sh <IP> [user]"
    exit 1
fi

print_status "Deploying to Contabo VDS: $SERVER_IP"
print_status "User: $SERVER_USER"
print_status "Deploy path: $DEPLOY_PATH"

# Check if we can connect to server
print_status "Testing SSH connection..."
if ! ssh -o ConnectTimeout=10 -o BatchMode=yes $SERVER_USER@$SERVER_IP exit 2>/dev/null; then
    print_error "Cannot connect to server. Please check:"
    echo "  1. Server IP is correct"
    echo "  2. SSH key is configured"
    echo "  3. Server is running"
    exit 1
fi
print_success "SSH connection successful"

# Create deployment package
print_status "Creating deployment package..."
TEMP_DIR=$(mktemp -d)
PACKAGE_NAME="sniperbot-$(date +%Y%m%d-%H%M%S).tar.gz"

# Copy necessary files
cp -r src/ $TEMP_DIR/
cp -r crates/ $TEMP_DIR/
cp -r configs/ $TEMP_DIR/
cp Cargo.toml $TEMP_DIR/
cp Dockerfile $TEMP_DIR/
cp docker-compose.production.yml $TEMP_DIR/
cp .env.production $TEMP_DIR/
cp contabo-setup.sh $TEMP_DIR/

# Create archive
cd $TEMP_DIR
tar -czf $PACKAGE_NAME *
mv $PACKAGE_NAME /tmp/
cd - > /dev/null
rm -rf $TEMP_DIR

print_success "Package created: /tmp/$PACKAGE_NAME"

# Upload package to server
print_status "Uploading package to Contabo VDS..."
scp /tmp/$PACKAGE_NAME $SERVER_USER@$SERVER_IP:/tmp/

# Execute deployment on server
print_status "Executing deployment on server..."
ssh $SERVER_USER@$SERVER_IP << EOF
set -e

echo "🔧 Setting up SniperBot on Contabo VDS..."

# Create deployment directory
mkdir -p $DEPLOY_PATH
cd $DEPLOY_PATH

# Extract package
tar -xzf /tmp/$PACKAGE_NAME

# Make setup script executable
chmod +x contabo-setup.sh

# Run setup
./contabo-setup.sh

echo "✅ Deployment completed!"
EOF

print_success "🎉 SniperBot 2.0 deployed successfully to Contabo VDS!"
print_status "Server: http://$SERVER_IP:8084"
print_status "Dashboard: http://$SERVER_IP:8084"

# Cleanup
rm -f /tmp/$PACKAGE_NAME

echo ""
echo "📋 Next steps:"
echo "  1. SSH to server: ssh $SERVER_USER@$SERVER_IP"
echo "  2. Check logs: docker logs sniperbot-production"
echo "  3. Monitor: docker stats"
echo "  4. Access dashboard: http://$SERVER_IP:8084"
echo ""
print_success "Deployment complete! 🚀"

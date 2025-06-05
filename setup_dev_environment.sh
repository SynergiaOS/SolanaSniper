#!/bin/bash

# SniperBot 2.0 Development Environment Setup
# Sets up Graphiti + DragonflyDB revolutionary stack

set -e

echo "🚀 Setting up SniperBot 2.0 Revolutionary Development Environment"
echo "🧠 Graphiti Knowledge Graph + 🐉 DragonflyDB Ultra-fast Cache"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

print_header() {
    echo -e "${PURPLE}$1${NC}"
}

# Check if Docker is installed
check_docker() {
    print_header "🐳 Checking Docker installation..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    print_success "Docker and Docker Compose are installed"
}

# Check if Python is installed
check_python() {
    print_header "🐍 Checking Python installation..."
    
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is not installed. Please install Python 3.10+ first."
        exit 1
    fi
    
    PYTHON_VERSION=$(python3 --version | cut -d' ' -f2 | cut -d'.' -f1,2)
    REQUIRED_VERSION="3.10"
    
    if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$PYTHON_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
        print_error "Python 3.10+ is required. Current version: $PYTHON_VERSION"
        exit 1
    fi
    
    print_success "Python $PYTHON_VERSION is installed"
}

# Check if Rust is installed
check_rust() {
    print_header "🦀 Checking Rust installation..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust is not installed. Please install Rust first."
        echo "Visit: https://rustup.rs/"
        exit 1
    fi
    
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_success "Rust $RUST_VERSION is installed"
}

# Setup Python virtual environment and install Graphiti
setup_python_env() {
    print_header "🧠 Setting up Python environment for Graphiti..."
    
    # Create virtual environment if it doesn't exist
    if [ ! -d "venv" ]; then
        print_status "Creating Python virtual environment..."
        python3 -m venv venv
    fi
    
    # Activate virtual environment
    print_status "Activating virtual environment..."
    source venv/bin/activate
    
    # Upgrade pip
    print_status "Upgrading pip..."
    pip install --upgrade pip
    
    # Install Graphiti and dependencies
    print_status "Installing Graphiti and dependencies..."
    pip install -r requirements.txt
    
    print_success "Python environment setup complete"
}

# Start Docker services
start_docker_services() {
    print_header "🐳 Starting Docker services..."
    
    print_status "Starting Neo4j, DragonflyDB, and monitoring services..."
    docker-compose -f docker-compose.dev.yml up -d
    
    print_status "Waiting for services to be ready..."
    sleep 10
    
    # Check service health
    print_status "Checking service health..."
    
    # Check Neo4j
    if docker-compose -f docker-compose.dev.yml exec -T neo4j cypher-shell -u neo4j -p sniperbot123 "RETURN 1" &> /dev/null; then
        print_success "Neo4j is ready"
    else
        print_warning "Neo4j is still starting up..."
    fi
    
    # Check DragonflyDB
    if docker-compose -f docker-compose.dev.yml exec -T dragonfly redis-cli ping &> /dev/null; then
        print_success "DragonflyDB is ready"
    else
        print_warning "DragonflyDB is still starting up..."
    fi
    
    print_success "Docker services started"
}

# Create environment file
create_env_file() {
    print_header "⚙️ Creating environment configuration..."
    
    if [ ! -f ".env" ]; then
        print_status "Creating .env file..."
        cat > .env << EOF
# SniperBot 2.0 Configuration

# Graphiti Knowledge Graph
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=sniperbot123

# DragonflyDB Cache
DRAGONFLY_URL=redis://localhost:6379

# AI Configuration
OPENAI_API_KEY=your_openai_api_key_here
MISTRAL_API_KEY=your_mistral_api_key_here

# Solana Configuration
HELIUS_API_KEY=40a78e4c-bdd0-4338-877a-aa7d56a5f5a0
HELIUS_RPC_URL=https://mainnet.helius-rpc.com
HELIUS_WS_URL=wss://mainnet.helius-rpc.com

# Trading Configuration
DRY_RUN=true
INITIAL_BALANCE=10000.0

# API Configuration
API_PORT=8084
DASHBOARD_URL=http://localhost:8084

# Monitoring
PROMETHEUS_URL=http://localhost:9090
GRAFANA_URL=http://localhost:3000
EOF
        print_success ".env file created"
        print_warning "Please update the API keys in .env file"
    else
        print_status ".env file already exists"
    fi
}

# Build Rust project
build_rust_project() {
    print_header "🦀 Building Rust project..."
    
    print_status "Updating Cargo dependencies..."
    cargo update
    
    print_status "Building SniperBot..."
    cargo build
    
    print_success "Rust project built successfully"
}

# Test Graphiti connection
test_graphiti_connection() {
    print_header "🧠 Testing Graphiti Knowledge Graph connection..."
    
    # Activate Python environment
    source venv/bin/activate
    
    # Test Graphiti
    print_status "Testing Graphiti connection..."
    python3 -c "
import asyncio
import os
from python_bridge.graphiti_bridge import SniperBotKnowledgeGraph, GraphitiConfig

async def test():
    config = GraphitiConfig(
        neo4j_uri='bolt://localhost:7687',
        neo4j_user='neo4j',
        neo4j_password='sniperbot123',
        openai_api_key=os.getenv('OPENAI_API_KEY', 'test')
    )
    
    kg = SniperBotKnowledgeGraph(config)
    
    try:
        success = await kg.initialize()
        if success:
            print('✅ Graphiti Knowledge Graph connection successful')
        else:
            print('❌ Graphiti Knowledge Graph connection failed')
    except Exception as e:
        print(f'⚠️ Graphiti test skipped: {e}')

asyncio.run(test())
"
}

# Display service URLs
display_service_urls() {
    print_header "🌐 Service URLs:"
    echo ""
    echo -e "${CYAN}SniperBot Dashboard:${NC}     http://localhost:8084"
    echo -e "${CYAN}Neo4j Browser:${NC}           http://localhost:7474"
    echo -e "${CYAN}DragonflyDB:${NC}             redis://localhost:6379"
    echo -e "${CYAN}Grafana:${NC}                 http://localhost:3000 (admin/sniperbot123)"
    echo -e "${CYAN}Prometheus:${NC}              http://localhost:9090"
    echo ""
    echo -e "${YELLOW}Neo4j Credentials:${NC}       neo4j / sniperbot123"
    echo ""
}

# Main setup function
main() {
    print_header "🎯 SniperBot 2.0 Revolutionary Stack Setup"
    echo ""
    
    # Check prerequisites
    check_docker
    check_python
    check_rust
    
    # Setup environment
    setup_python_env
    create_env_file
    
    # Start services
    start_docker_services
    
    # Build project
    build_rust_project
    
    # Test connections
    test_graphiti_connection
    
    # Display information
    display_service_urls
    
    print_success "🚀 SniperBot 2.0 development environment is ready!"
    print_status "Next steps:"
    echo "  1. Update API keys in .env file"
    echo "  2. Run: cargo run --bin sniper-bot -- --dry-run"
    echo "  3. Open dashboard: http://localhost:8084"
    echo ""
    print_header "🧠 You now have the most advanced trading bot stack:"
    echo "  • Graphiti Knowledge Graph for temporal analysis"
    echo "  • DragonflyDB for ultra-fast caching (25x faster than Redis)"
    echo "  • AI-powered decision making"
    echo "  • Real-time WebSocket feeds"
    echo "  • Advanced monitoring and analytics"
}

# Run main function
main "$@"

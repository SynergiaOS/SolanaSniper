# 🚀 SniperBot 2.0 - Deployment Guide

This document provides comprehensive deployment instructions for SniperBot 2.0 across different environments.

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development](#local-development)
- [Docker Deployment](#docker-deployment)
- [Production Deployment](#production-deployment)
- [Cloud Deployment](#cloud-deployment)
- [Monitoring Setup](#monitoring-setup)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)

## 📋 Prerequisites

### System Requirements

**Minimum Requirements:**
- CPU: 2 cores, 2.4 GHz
- RAM: 4 GB
- Storage: 20 GB SSD
- Network: Stable internet connection (< 100ms latency)

**Recommended Requirements:**
- CPU: 4+ cores, 3.0+ GHz
- RAM: 8+ GB
- Storage: 50+ GB NVMe SSD
- Network: Low-latency connection (< 50ms)

### Software Dependencies

- **Rust**: 1.75+
- **Docker**: 20.10+
- **Docker Compose**: 2.0+
- **Git**: 2.30+

### API Keys Required

- **Helius API Key**: For Solana RPC access
- **Exchange API Keys**: For CEX integration (optional)

## 💻 Local Development

### Quick Start

```bash
# Clone repository
git clone https://github.com/SynergiaOS/SniperBot.git
cd SniperBot

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Copy configuration template
cp config.example.toml config.toml

# Edit configuration with your API keys
nano config.toml

# Build and run in development mode
cargo build
cargo run -- --dry-run --log-level debug
```

### Development Configuration

```toml
[bot]
environment = "development"
dry_run = true
update_interval_ms = 1000

[logging]
level = "debug"
file_path = "logs/dev.log"

[api]
auth_enabled = false
port = 8080
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test data_fetcher

# Run with output
cargo test -- --nocapture

# Generate coverage report
cargo tarpaulin --out Html
```

## 🐳 Docker Deployment

### Single Container

```bash
# Build Docker image
docker build -t sniperbot:latest .

# Run container
docker run -d \
  --name sniperbot \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml \
  -v $(pwd)/logs:/app/logs \
  -v $(pwd)/data:/app/data \
  --env-file .env \
  sniperbot:latest
```

### Docker Compose (Recommended)

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  sniperbot:
    build: .
    container_name: sniperbot
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./logs:/app/logs
      - ./data:/app/data
      - ./certs:/app/certs:ro
    env_file:
      - .env
    depends_on:
      - redis
      - questdb
    networks:
      - sniperbot-network

  redis:
    image: redis:7-alpine
    container_name: sniperbot-redis
    restart: unless-stopped
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes
    networks:
      - sniperbot-network

  questdb:
    image: questdb/questdb:7.3.10
    container_name: sniperbot-questdb
    restart: unless-stopped
    ports:
      - "9000:9000"
      - "8812:8812"
    volumes:
      - questdb-data:/var/lib/questdb
    environment:
      - QDB_CAIRO_COMMIT_LAG=1000
    networks:
      - sniperbot-network

  prometheus:
    image: prom/prometheus:latest
    container_name: sniperbot-prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
    networks:
      - sniperbot-network

  grafana:
    image: grafana/grafana:latest
    container_name: sniperbot-grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=sniperbot123
    networks:
      - sniperbot-network

volumes:
  redis-data:
  questdb-data:
  prometheus-data:
  grafana-data:

networks:
  sniperbot-network:
    driver: bridge
```

### Deploy with Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f sniperbot

# Stop services
docker-compose down

# Update and restart
docker-compose pull
docker-compose up -d --force-recreate
```

## 🏭 Production Deployment

### Production Configuration

```toml
[bot]
environment = "production"
dry_run = false
update_interval_ms = 100
max_concurrent_orders = 10

[logging]
level = "info"
structured = true
file_path = "/var/log/sniperbot/app.log"

[api]
auth_enabled = true
tls_enabled = true
host = "0.0.0.0"
port = 8080

[database]
sqlite_path = "/var/lib/sniperbot/data.db"
redis_url = "redis://redis:6379"
questdb_url = "http://questdb:9000"
```

### Systemd Service

Create `/etc/systemd/system/sniperbot.service`:

```ini
[Unit]
Description=SniperBot 2.0 Trading Bot
After=network.target
Wants=network.target

[Service]
Type=simple
User=sniperbot
Group=sniperbot
WorkingDirectory=/opt/sniperbot
ExecStart=/opt/sniperbot/target/release/sniperbot --config /etc/sniperbot/config.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=sniperbot

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/sniperbot /var/log/sniperbot

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

### Installation Script

```bash
#!/bin/bash
# install.sh

set -e

# Create user and directories
sudo useradd -r -s /bin/false sniperbot
sudo mkdir -p /opt/sniperbot /etc/sniperbot /var/lib/sniperbot /var/log/sniperbot
sudo chown sniperbot:sniperbot /var/lib/sniperbot /var/log/sniperbot

# Build and install binary
cargo build --release
sudo cp target/release/sniperbot /opt/sniperbot/
sudo chown sniperbot:sniperbot /opt/sniperbot/sniperbot
sudo chmod +x /opt/sniperbot/sniperbot

# Install configuration
sudo cp config.toml /etc/sniperbot/
sudo chown sniperbot:sniperbot /etc/sniperbot/config.toml
sudo chmod 600 /etc/sniperbot/config.toml

# Install systemd service
sudo cp sniperbot.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable sniperbot

# Start service
sudo systemctl start sniperbot
sudo systemctl status sniperbot
```

## ☁️ Cloud Deployment

### AWS Deployment

#### EC2 Instance

```bash
# Launch EC2 instance (t3.medium recommended)
aws ec2 run-instances \
  --image-id ami-0abcdef1234567890 \
  --instance-type t3.medium \
  --key-name your-key-pair \
  --security-group-ids sg-12345678 \
  --subnet-id subnet-12345678 \
  --user-data file://user-data.sh
```

#### ECS Deployment

Create `task-definition.json`:

```json
{
  "family": "sniperbot",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "sniperbot",
      "image": "your-account.dkr.ecr.region.amazonaws.com/sniperbot:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "ENVIRONMENT",
          "value": "production"
        }
      ],
      "secrets": [
        {
          "name": "HELIUS_API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:sniperbot/helius-api-key"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/sniperbot",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

### Google Cloud Platform

#### Cloud Run Deployment

```bash
# Build and push to Container Registry
docker build -t gcr.io/your-project/sniperbot:latest .
docker push gcr.io/your-project/sniperbot:latest

# Deploy to Cloud Run
gcloud run deploy sniperbot \
  --image gcr.io/your-project/sniperbot:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --memory 2Gi \
  --cpu 2 \
  --max-instances 10
```

### Kubernetes Deployment

Create `k8s-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sniperbot
  labels:
    app: sniperbot
spec:
  replicas: 2
  selector:
    matchLabels:
      app: sniperbot
  template:
    metadata:
      labels:
        app: sniperbot
    spec:
      containers:
      - name: sniperbot
        image: sniperbot:latest
        ports:
        - containerPort: 8080
        env:
        - name: ENVIRONMENT
          value: "production"
        - name: HELIUS_API_KEY
          valueFrom:
            secretKeyRef:
              name: sniperbot-secrets
              key: helius-api-key
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: sniperbot-service
spec:
  selector:
    app: sniperbot
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

## 📊 Monitoring Setup

### Prometheus Configuration

Create `monitoring/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'sniperbot'
    static_configs:
      - targets: ['sniperbot:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']

  - job_name: 'questdb'
    static_configs:
      - targets: ['questdb:9000']
```

### Grafana Dashboards

Import pre-built dashboards:

```bash
# Copy dashboard configurations
cp -r monitoring/grafana/dashboards/* /var/lib/grafana/dashboards/

# Restart Grafana
docker-compose restart grafana
```

### Log Aggregation

#### ELK Stack

```yaml
# Add to docker-compose.yml
elasticsearch:
  image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
  environment:
    - discovery.type=single-node
    - xpack.security.enabled=false
  ports:
    - "9200:9200"

logstash:
  image: docker.elastic.co/logstash/logstash:8.11.0
  volumes:
    - ./monitoring/logstash.conf:/usr/share/logstash/pipeline/logstash.conf
  depends_on:
    - elasticsearch

kibana:
  image: docker.elastic.co/kibana/kibana:8.11.0
  ports:
    - "5601:5601"
  depends_on:
    - elasticsearch
```

## 🔒 Security Considerations

### API Security

```toml
[api]
auth_enabled = true
api_key = "your-very-secure-api-key-here"
rate_limit_per_minute = 100
max_request_size_mb = 1

[api.tls]
enabled = true
cert_file = "/etc/ssl/certs/sniperbot.crt"
key_file = "/etc/ssl/private/sniperbot.key"
```

### Firewall Configuration

```bash
# UFW configuration
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 8080/tcp  # API port
sudo ufw allow 3000/tcp  # Grafana (if needed)
sudo ufw enable
```

### Secret Management

#### Docker Secrets

```bash
# Create secrets
echo "your-helius-api-key" | docker secret create helius_api_key -
echo "your-binance-api-key" | docker secret create binance_api_key -

# Use in docker-compose.yml
services:
  sniperbot:
    secrets:
      - helius_api_key
      - binance_api_key

secrets:
  helius_api_key:
    external: true
  binance_api_key:
    external: true
```

### Backup Strategy

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backup/sniperbot/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup database
cp /var/lib/sniperbot/data.db "$BACKUP_DIR/"

# Backup configuration
cp /etc/sniperbot/config.toml "$BACKUP_DIR/"

# Backup logs (last 7 days)
find /var/log/sniperbot -name "*.log" -mtime -7 -exec cp {} "$BACKUP_DIR/" \;

# Compress backup
tar -czf "$BACKUP_DIR.tar.gz" -C "$(dirname "$BACKUP_DIR")" "$(basename "$BACKUP_DIR")"
rm -rf "$BACKUP_DIR"

# Upload to S3 (optional)
aws s3 cp "$BACKUP_DIR.tar.gz" s3://your-backup-bucket/sniperbot/
```

## 🔧 Troubleshooting

### Common Issues

#### High Memory Usage

```bash
# Check memory usage
docker stats sniperbot

# Adjust memory limits in docker-compose.yml
services:
  sniperbot:
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 1G
```

#### Connection Issues

```bash
# Check network connectivity
docker exec sniperbot ping api.helius.xyz

# Check DNS resolution
docker exec sniperbot nslookup api.helius.xyz

# Check port availability
netstat -tlnp | grep 8080
```

#### Performance Issues

```bash
# Check CPU usage
top -p $(pgrep sniperbot)

# Check disk I/O
iotop -p $(pgrep sniperbot)

# Check network I/O
nethogs
```

### Log Analysis

```bash
# View real-time logs
docker-compose logs -f sniperbot

# Search for errors
docker-compose logs sniperbot | grep ERROR

# Check specific time range
docker-compose logs --since="2024-01-15T10:00:00" --until="2024-01-15T11:00:00" sniperbot
```

### Health Checks

```bash
# API health check
curl http://localhost:8080/health

# Detailed status
curl http://localhost:8080/api/v1/status

# Check data sources
curl http://localhost:8080/api/v1/data-sources
```

---

**For more information, see the [Configuration Guide](configuration.md) and [API Documentation](api.md).**

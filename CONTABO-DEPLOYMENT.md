# 🚀 SniperBot 2.0 - Contabo VDS Deployment Guide

## 📋 Prerequisites

### 1. Contabo VDS Server
- **RAM:** 24GB (recommended)
- **CPU:** Multi-core
- **Storage:** SSD
- **OS:** Ubuntu 20.04+ or Debian 11+

### 2. Local Requirements
- SSH access to Contabo server
- SSH key configured for passwordless login

## 🔧 Quick Deployment

### Step 1: Configure SSH Access
```bash
# Add your SSH key to the server
ssh-copy-id root@YOUR_CONTABO_IP

# Test connection
ssh root@YOUR_CONTABO_IP
```

### Step 2: Deploy SniperBot
```bash
# Run deployment script
./contabo-deploy.sh YOUR_CONTABO_IP root

# Example:
./contabo-deploy.sh 192.168.1.100 root
```

### Step 3: Verify Deployment
```bash
# SSH to server
ssh root@YOUR_CONTABO_IP

# Check status
docker ps
docker logs sniperbot-production

# Test API
curl http://localhost:8084/health
curl http://localhost:8084/api/bot-status
```

## 📊 Service URLs

After deployment, access these URLs (replace IP):

- **SniperBot API:** `http://YOUR_CONTABO_IP:8084`
- **Dashboard:** `http://YOUR_CONTABO_IP:8084`
- **Health Check:** `http://YOUR_CONTABO_IP:8084/health`

## 🛠️ Management Commands

### On Contabo Server:
```bash
# View logs
docker logs -f sniperbot-production

# Restart bot
docker-compose -f docker-compose.production.yml restart sniper-bot

# Stop bot
docker-compose -f docker-compose.production.yml down

# Start bot
docker-compose -f docker-compose.production.yml up -d

# View status
docker-compose -f docker-compose.production.yml ps

# System monitoring
htop
docker stats
```

## 🔒 Security Configuration

The deployment automatically configures:

- **Firewall (UFW):**
  - SSH (22) - allowed
  - SniperBot API (8084) - allowed
  - HTTP (80) - allowed
  - HTTPS (443) - allowed
  - All other ports - denied

- **Fail2Ban:** Protection against brute force attacks

## 📈 Monitoring

### Real-time Monitoring:
```bash
# System resources
htop

# Docker containers
docker stats

# Bot logs
docker logs -f sniperbot-production

# Network connections
netstat -tulpn | grep :8084
```

### Log Files:
- **Bot logs:** `/opt/sniperbot/logs/`
- **Docker logs:** `docker logs sniperbot-production`
- **System logs:** `/var/log/syslog`

## 🚨 Important Notes

### ⚠️ LIVE TRADING WARNING
- **Bot is in LIVE TRADING mode with real money!**
- **Monitor logs regularly**
- **Set up alerts for critical events**
- **Keep private keys secure**

### 💰 Trading Configuration
- **Wallet:** 0.1 SOL (~$2.00)
- **Trade Size:** 0.05 SOL per transaction
- **Active Strategy:** PumpFun Sniping only (low balance mode)
- **MEV Protection:** Jito bundles enabled
- **AI Decision Engine:** Mistral AI enabled

### 🔐 Security Best Practices
1. **Change default passwords**
2. **Use SSH keys only (disable password auth)**
3. **Keep system updated**
4. **Monitor access logs**
5. **Backup wallet keys securely**

## 🔧 Troubleshooting

### Bot Not Starting:
```bash
# Check logs
docker logs sniperbot-production

# Check configuration
cat /opt/sniperbot/.env.production

# Rebuild if needed
cd /opt/sniperbot
docker-compose -f docker-compose.production.yml build sniper-bot
docker-compose -f docker-compose.production.yml up -d
```

### API Not Responding:
```bash
# Check if port is open
netstat -tulpn | grep :8084

# Check firewall
ufw status

# Test locally
curl http://localhost:8084/health
```

### High Resource Usage:
```bash
# Monitor resources
htop
docker stats

# Check disk space
df -h

# Clean up if needed
docker system prune -f
```

## 📞 Support

If you encounter issues:

1. **Check logs first:** `docker logs sniperbot-production`
2. **Verify configuration:** `.env.production` file
3. **Test connectivity:** API endpoints
4. **Monitor resources:** CPU, RAM, disk usage

## 🎉 Success Indicators

✅ **Deployment Successful When:**
- Docker container is running
- API responds to health checks
- Bot logs show "LIVE TRADING" mode
- WebSocket connections established
- Portfolio monitoring active

**Example successful log output:**
```
🚀 Live Trading Engine: AKTYWNY
💰 Mode: LIVE TRADING
✅ Connected to Helius WebSocket
✅ Connected to Binance WebSocket
💰 Portfolio Status: 0.1 SOL (✅), 0 tokens, Total: $2.00
```

---

**🚀 Ready to make money with SniperBot 2.0 on Contabo VDS!**

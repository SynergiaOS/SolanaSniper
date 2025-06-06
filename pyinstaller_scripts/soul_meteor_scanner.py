#!/usr/bin/env python3
"""
🕸️ Soul Meteor Scanner - SniperBot 2.0
Real-time DLMM pool scanner using Meteora API

This scanner finds "hot" new liquidity pools based on:
- High liquidity (>$20k)
- Recent creation (age filtering)
- High volume activity
- Fee/TVL ratio analysis
"""

import requests
import json
import time
import redis
import sys
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional

# --- CONFIGURATION ---
class ScannerConfig:
    # API Configuration
    METEORA_API_BASE = "https://dlmm-api.meteora.ag"
    REQUEST_TIMEOUT = 15

    # Filter Criteria (based on research)
    MIN_LIQUIDITY_USD = 20000  # Minimum $20k liquidity
    MIN_VOLUME_24H_USD = 5000  # Minimum $5k volume in 24h
    MAX_AGE_HOURS = 24         # Only pools created in last 24h
    MIN_FEE_TVL_RATIO = 0.01   # Minimum 1% fee/TVL ratio (24h)

    # Sorting and Limits
    SORT_BY = "volume"         # Sort by volume (most active first)
    MAX_RESULTS = 50           # Maximum pools to analyze

    # Headers for requests
    HEADERS = {
        'User-Agent': 'SoulMeteorScanner/1.0 (SniperBot 2.0)',
        'Accept': 'application/json',
        'Content-Type': 'application/json'
    }

    # DragonflyDB Configuration
    DRAGONFLY_URL = "redis://localhost:6379"  # Default local Redis/DragonflyDB
    DRAGONFLY_PASSWORD = None  # Set if authentication required
    OPPORTUNITY_TTL_HOURS = 2  # How long to keep opportunities in DB
    ENABLE_DATABASE = True  # Enable/disable database operations

class DragonflyDBManager:
    """Manages DragonflyDB connections and operations"""

    def __init__(self, config: ScannerConfig):
        self.config = config
        self.redis_client = None

    def connect(self):
        """Connect to DragonflyDB"""
        try:
            self.redis_client = redis.Redis.from_url(
                self.config.DRAGONFLY_URL,
                password=self.config.DRAGONFLY_PASSWORD,
                decode_responses=True
            )
            # Test connection
            self.redis_client.ping()
            print("✅ Connected to DragonflyDB", file=sys.stderr)
            return True
        except Exception as e:
            print(f"❌ Failed to connect to DragonflyDB: {e}", file=sys.stderr)
            return False

    def save_raw_opportunities(self, opportunities: List[Dict[str, Any]]) -> int:
        """Save raw opportunities to DragonflyDB"""
        if not self.redis_client:
            print("⚠️ No DragonflyDB connection", file=sys.stderr)
            return 0

        saved_count = 0
        pipeline = self.redis_client.pipeline()

        for opp in opportunities:
            try:
                # Create raw opportunity record
                raw_opportunity = {
                    "id": f"{opp['address']}_{int(datetime.now().timestamp())}",
                    "candidate": opp,
                    "discovered_at": datetime.now().isoformat(),
                    "expires_at": (datetime.now() + timedelta(hours=self.config.OPPORTUNITY_TTL_HOURS)).isoformat(),
                    "status": "Pending"
                }

                # Store with TTL
                key = f"raw_opportunity:{opp['address']}"
                pipeline.set(
                    key,
                    json.dumps(raw_opportunity),
                    ex=self.config.OPPORTUNITY_TTL_HOURS * 3600
                )

                # Add to list of all raw opportunities
                pipeline.lpush("all_raw_opportunities", key)

                # Add to processed tokens set (for deduplication)
                pipeline.sadd("processed_tokens", opp['address'])
                pipeline.expire("processed_tokens", 3600)  # 1 hour TTL

                saved_count += 1

            except Exception as e:
                print(f"⚠️ Failed to prepare opportunity {opp.get('name', 'Unknown')}: {e}", file=sys.stderr)

        try:
            pipeline.execute()
            print(f"💾 Saved {saved_count} opportunities to DragonflyDB", file=sys.stderr)
            return saved_count
        except Exception as e:
            print(f"❌ Failed to save opportunities to DragonflyDB: {e}", file=sys.stderr)
            return 0

    def is_token_recently_processed(self, token_address: str) -> bool:
        """Check if token was recently processed (deduplication)"""
        if not self.redis_client:
            return False
        try:
            return self.redis_client.sismember("processed_tokens", token_address)
        except Exception:
            return False

class PoolAnalyzer:
    """Analyzes individual pool data for trading opportunities"""
    
    @staticmethod
    def calculate_pool_age_hours(pool_data: Dict[str, Any]) -> Optional[float]:
        """Calculate pool age in hours (simplified - would need creation timestamp)"""
        # Note: Meteora API doesn't provide creation timestamp directly
        # This would need to be enhanced with on-chain data or additional API calls
        return None
    
    @staticmethod
    def extract_pool_metrics(pool_data: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Extract key metrics from pool data"""
        try:
            # Basic pool info
            address = pool_data.get('address', '')
            name = pool_data.get('name', 'Unknown')
            
            # Financial metrics
            liquidity_str = pool_data.get('liquidity', '0')
            liquidity_usd = float(liquidity_str) if liquidity_str.replace('.', '').isdigit() else 0.0
            
            volume_24h = pool_data.get('trade_volume_24h', 0.0)
            fees_24h = pool_data.get('fees_24h', 0.0)
            
            # Calculate fee/TVL ratio
            fee_tvl_ratio_24h = (fees_24h / liquidity_usd * 100) if liquidity_usd > 0 else 0.0
            
            # APR/APY
            apr = pool_data.get('apr', 0.0)
            apy = pool_data.get('apy', 0.0)
            
            # Token info
            mint_x = pool_data.get('mint_x', '')
            mint_y = pool_data.get('mint_y', '')
            current_price = pool_data.get('current_price', 0.0)
            
            return {
                'address': address,
                'name': name,
                'liquidity_usd': liquidity_usd,
                'volume_24h': volume_24h,
                'fees_24h': fees_24h,
                'fee_tvl_ratio_24h': fee_tvl_ratio_24h,
                'apr': apr,
                'apy': apy,
                'mint_x': mint_x,
                'mint_y': mint_y,
                'current_price': current_price,
                'is_blacklisted': pool_data.get('is_blacklisted', False),
                'hide': pool_data.get('hide', False)
            }
            
        except Exception as e:
            print(f"❌ Error extracting metrics for pool {pool_data.get('name', 'Unknown')}: {e}")
            return None

class SoulMeteorScanner:
    """Main scanner class for finding hot DLMM pools"""

    def __init__(self, config: Optional[ScannerConfig] = None):
        self.config = config or ScannerConfig()
        self.analyzer = PoolAnalyzer()
        self.db_manager = DragonflyDBManager(self.config)
        self.use_database = False  # Will be set based on connection success
        
    def fetch_all_pools(self) -> Optional[List[Dict[str, Any]]]:
        """Fetch all DLMM pools from Meteora API"""
        url = f"{self.config.METEORA_API_BASE}/pair/all"

        try:
            import sys
            print(f"🚀 Fetching pools from Meteora API...", file=sys.stderr)
            print(f"📊 URL: {url}", file=sys.stderr)

            response = requests.get(
                url,
                headers=self.config.HEADERS,
                timeout=self.config.REQUEST_TIMEOUT
            )
            response.raise_for_status()

            pools = response.json()

            print(f"✅ Successfully fetched {len(pools)} pools", file=sys.stderr)

            # Sort by volume (descending) and limit results
            pools_with_volume = [p for p in pools if p.get('trade_volume_24h', 0) > 0]
            sorted_pools = sorted(pools_with_volume, key=lambda x: x.get('trade_volume_24h', 0), reverse=True)
            limited_pools = sorted_pools[:self.config.MAX_RESULTS]

            print(f"📊 Filtered to top {len(limited_pools)} pools by volume", file=sys.stderr)
            return limited_pools
            
        except requests.exceptions.RequestException as e:
            import sys
            print(f"❌ Network error fetching pools: {e}", file=sys.stderr)
            return None
        except json.JSONDecodeError as e:
            import sys
            print(f"❌ JSON decode error: {e}", file=sys.stderr)
            return None
        except Exception as e:
            import sys
            print(f"❌ Unexpected error: {e}", file=sys.stderr)
            return None
    
    def apply_filters(self, pools: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Apply trading filters to find hot opportunities"""
        hot_pools = []
        
        import sys
        print(f"\n🔍 Applying filters to {len(pools)} pools...", file=sys.stderr)
        print(f"📋 Filter criteria:", file=sys.stderr)
        print(f"   • Min Liquidity: ${self.config.MIN_LIQUIDITY_USD:,}", file=sys.stderr)
        print(f"   • Min Volume 24h: ${self.config.MIN_VOLUME_24H_USD:,}", file=sys.stderr)
        print(f"   • Min Fee/TVL Ratio: {self.config.MIN_FEE_TVL_RATIO:.2%}", file=sys.stderr)
        
        for pool in pools:
            metrics = self.analyzer.extract_pool_metrics(pool)
            if not metrics:
                continue
            
            # Skip blacklisted or hidden pools
            if metrics['is_blacklisted'] or metrics['hide']:
                continue
            
            # Apply filters
            passes_liquidity = metrics['liquidity_usd'] >= self.config.MIN_LIQUIDITY_USD
            passes_volume = metrics['volume_24h'] >= self.config.MIN_VOLUME_24H_USD
            passes_fee_ratio = metrics['fee_tvl_ratio_24h'] >= self.config.MIN_FEE_TVL_RATIO
            
            if passes_liquidity and passes_volume and passes_fee_ratio:
                # Calculate opportunity score
                opportunity_score = self._calculate_opportunity_score(metrics)
                metrics['opportunity_score'] = opportunity_score
                
                hot_pools.append(metrics)

                import sys
                print(f"🔥 HOT POOL FOUND: {metrics['name']}", file=sys.stderr)
                print(f"   💰 Liquidity: ${metrics['liquidity_usd']:,.2f}", file=sys.stderr)
                print(f"   📊 Volume 24h: ${metrics['volume_24h']:,.2f}", file=sys.stderr)
                print(f"   💸 Fee/TVL: {metrics['fee_tvl_ratio_24h']:.2%}", file=sys.stderr)
                print(f"   ⭐ Score: {opportunity_score:.2f}", file=sys.stderr)
        
        # Sort by opportunity score
        hot_pools.sort(key=lambda x: x['opportunity_score'], reverse=True)

        import sys
        print(f"\n✅ Found {len(hot_pools)} hot pools after filtering", file=sys.stderr)
        return hot_pools
    
    def _calculate_opportunity_score(self, metrics: Dict[str, Any]) -> float:
        """Calculate opportunity score based on multiple factors"""
        score = 0.0
        
        # Volume factor (higher volume = higher score)
        volume_factor = min(metrics['volume_24h'] / 100000, 5.0)  # Cap at 5x
        score += volume_factor * 0.3
        
        # Fee/TVL ratio factor (higher ratio = higher score)
        fee_ratio_factor = min(metrics['fee_tvl_ratio_24h'] * 10, 5.0)  # Cap at 5x
        score += fee_ratio_factor * 0.4
        
        # APR factor (higher APR = higher score)
        apr_factor = min(metrics['apr'] / 100, 3.0)  # Cap at 3x
        score += apr_factor * 0.2
        
        # Liquidity factor (more liquidity = slightly higher score, but diminishing returns)
        liquidity_factor = min(metrics['liquidity_usd'] / 1000000, 2.0)  # Cap at 2x
        score += liquidity_factor * 0.1
        
        return round(score, 2)
    
    def scan_for_opportunities(self) -> List[Dict[str, Any]]:
        """Main scanning function - returns list of hot opportunities"""
        print("🕸️ Soul Meteor Scanner - Starting scan...", file=sys.stderr)
        print(f"⏰ Scan time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}", file=sys.stderr)

        # Try to connect to DragonflyDB
        self.use_database = self.db_manager.connect()

        # Fetch pools
        pools = self.fetch_all_pools()
        if not pools:
            print("❌ Failed to fetch pools", file=sys.stderr)
            return []

        # Apply filters
        hot_opportunities = self.apply_filters(pools)

        # Save to DragonflyDB if connected
        if self.use_database and hot_opportunities:
            saved_count = self.db_manager.save_raw_opportunities(hot_opportunities)
            print(f"💾 Saved {saved_count}/{len(hot_opportunities)} opportunities to DragonflyDB", file=sys.stderr)

        return hot_opportunities
    
    def print_summary(self, opportunities: List[Dict[str, Any]]):
        """Print summary of found opportunities"""
        import sys
        if not opportunities:
            print("\n📊 SCAN SUMMARY: No opportunities found matching criteria", file=sys.stderr)
            return

        print(f"\n📊 SCAN SUMMARY: {len(opportunities)} HOT OPPORTUNITIES", file=sys.stderr)
        print("=" * 80, file=sys.stderr)
        
        import sys
        for i, opp in enumerate(opportunities[:10], 1):  # Top 10
            print(f"\n🏆 #{i} - {opp['name']}", file=sys.stderr)
            print(f"   📍 Address: {opp['address'][:8]}...{opp['address'][-8:]}", file=sys.stderr)
            print(f"   💰 Liquidity: ${opp['liquidity_usd']:,.2f}", file=sys.stderr)
            print(f"   📊 Volume 24h: ${opp['volume_24h']:,.2f}", file=sys.stderr)
            print(f"   💸 Fees 24h: ${opp['fees_24h']:,.2f}", file=sys.stderr)
            print(f"   📈 Fee/TVL: {opp['fee_tvl_ratio_24h']:.2%}", file=sys.stderr)
            print(f"   🎯 APR: {opp['apr']:.1f}%", file=sys.stderr)
            print(f"   ⭐ Score: {opp['opportunity_score']:.2f}", file=sys.stderr)

def main():
    """Main entry point"""
    import sys

    # All logs go to stderr, only JSON goes to stdout
    print("🚀 Soul Meteor Scanner - SniperBot 2.0", file=sys.stderr)
    print("=" * 50, file=sys.stderr)

    # Initialize scanner
    scanner = SoulMeteorScanner()

    # Scan for opportunities
    opportunities = scanner.scan_for_opportunities()

    # Print summary to stderr
    scanner.print_summary(opportunities)

    # Convert opportunities to JSON format for Rust
    json_output = []
    for opp in opportunities:
        json_output.append({
            "name": opp['name'],
            "address": opp['address'],
            "liquidity_usd": opp['liquidity_usd'],
            "volume_24h": opp['volume_24h'],
            "fees_24h": opp['fees_24h'],
            "fee_tvl_ratio_24h": opp['fee_tvl_ratio_24h'],
            "apr": opp['apr'],
            "apy": opp['apy'],
            "opportunity_score": opp['opportunity_score'],
            "mint_x": opp['mint_x'],
            "mint_y": opp['mint_y'],
            "current_price": opp['current_price']
        })

    # Output clean JSON to stdout for Rust consumption
    print(json.dumps(json_output, indent=None), file=sys.stdout)

    return opportunities

if __name__ == "__main__":
    opportunities = main()

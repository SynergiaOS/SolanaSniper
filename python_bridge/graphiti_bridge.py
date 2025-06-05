"""
Graphiti Knowledge Graph Bridge for SniperBot 2.0
Provides Python interface for building temporal trading knowledge graphs
"""

import asyncio
import json
import os
from datetime import datetime
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from pydantic import BaseModel

from graphiti_core import Graphiti
from graphiti_core.nodes import Node as BaseNode
from graphiti_core.edges import Edge as BaseEdge


# Custom Trading Entities for SniperBot Knowledge Graph
class Token(BaseNode):
    """Cryptocurrency token entity"""
    symbol: str
    name: Optional[str] = None
    price: float
    volume_24h: Optional[float] = None
    market_cap: Optional[float] = None
    chain: str = "solana"
    contract_address: Optional[str] = None


class Strategy(BaseNode):
    """Trading strategy entity"""
    name: str
    strategy_type: str  # arbitrage, sniping, dlmm, etc.
    performance: float = 0.0
    active: bool = True
    win_rate: float = 0.0
    total_signals: int = 0


class Signal(BaseNode):
    """Trading signal entity"""
    strength: float
    confidence: float
    signal_type: str  # buy, sell, hold
    executed: bool = False
    profit_loss: Optional[float] = None


class MarketEvent(BaseNode):
    """Market event entity"""
    event_type: str  # price_update, volume_spike, whale_movement
    impact: float
    source: str  # binance, helius, jupiter
    metadata: Dict[str, Any] = {}


class Exchange(BaseNode):
    """Exchange/DEX entity"""
    name: str
    exchange_type: str  # cex, dex
    volume_24h: Optional[float] = None
    fees: Optional[float] = None


# Custom Relationships
class PriceCorrelation(BaseEdge):
    """Price correlation between tokens"""
    correlation_coefficient: float
    timeframe: str  # 1h, 24h, 7d
    strength: str  # weak, moderate, strong


class GeneratesSignal(BaseEdge):
    """Strategy generates signal"""
    timestamp: datetime
    conditions_met: List[str]


class AffectsPrice(BaseEdge):
    """Market event affects token price"""
    impact_percentage: float
    duration_minutes: Optional[int] = None


class TradedOn(BaseEdge):
    """Token traded on exchange"""
    volume_24h: Optional[float] = None
    liquidity: Optional[float] = None


@dataclass
class GraphitiConfig:
    """Configuration for Graphiti Knowledge Graph"""
    neo4j_uri: str = "bolt://localhost:7687"
    neo4j_user: str = "neo4j"
    neo4j_password: str = "password"
    openai_api_key: Optional[str] = None
    embedding_model: str = "text-embedding-3-small"
    llm_model: str = "gpt-4o-mini"


class SniperBotKnowledgeGraph:
    """Main Knowledge Graph manager for SniperBot"""
    
    def __init__(self, config: GraphitiConfig):
        self.config = config
        self.graphiti: Optional[Graphiti] = None
        self._initialized = False
    
    async def initialize(self) -> bool:
        """Initialize Graphiti connection and schema"""
        try:
            # Set OpenAI API key if provided
            if self.config.openai_api_key:
                os.environ["OPENAI_API_KEY"] = self.config.openai_api_key
            
            # Initialize Graphiti
            self.graphiti = Graphiti(
                self.config.neo4j_uri,
                self.config.neo4j_user,
                self.config.neo4j_password
            )
            
            # Test connection
            await self.graphiti.build_indices_and_constraints()
            
            self._initialized = True
            print("✅ Graphiti Knowledge Graph initialized successfully")
            return True
            
        except Exception as e:
            print(f"❌ Failed to initialize Graphiti: {e}")
            return False
    
    async def add_token(self, symbol: str, price: float, **kwargs) -> bool:
        """Add or update token in knowledge graph"""
        if not self._initialized:
            return False
        
        try:
            token = Token(
                symbol=symbol,
                price=price,
                **kwargs
            )
            
            episode_data = {
                "action": "token_update",
                "token": token.dict(),
                "timestamp": datetime.now().isoformat()
            }
            
            await self.graphiti.add_episode(
                name=f"token_update_{symbol}",
                episode_body=json.dumps(episode_data),
                source_description=f"Token price update for {symbol}"
            )
            
            return True
            
        except Exception as e:
            print(f"❌ Failed to add token {symbol}: {e}")
            return False
    
    async def add_signal(self, strategy_name: str, symbol: str, strength: float, 
                        signal_type: str, **kwargs) -> bool:
        """Add trading signal to knowledge graph"""
        if not self._initialized:
            return False
        
        try:
            signal_data = {
                "action": "signal_generated",
                "strategy": strategy_name,
                "symbol": symbol,
                "strength": strength,
                "signal_type": signal_type,
                "timestamp": datetime.now().isoformat(),
                **kwargs
            }
            
            await self.graphiti.add_episode(
                name=f"signal_{strategy_name}_{symbol}",
                episode_body=json.dumps(signal_data),
                source_description=f"Trading signal from {strategy_name} for {symbol}"
            )
            
            return True
            
        except Exception as e:
            print(f"❌ Failed to add signal: {e}")
            return False
    
    async def add_market_event(self, event_type: str, symbol: str, impact: float,
                              source: str, **kwargs) -> bool:
        """Add market event to knowledge graph"""
        if not self._initialized:
            return False
        
        try:
            event_data = {
                "action": "market_event",
                "event_type": event_type,
                "symbol": symbol,
                "impact": impact,
                "source": source,
                "timestamp": datetime.now().isoformat(),
                **kwargs
            }
            
            await self.graphiti.add_episode(
                name=f"event_{event_type}_{symbol}",
                episode_body=json.dumps(event_data),
                source_description=f"Market event: {event_type} for {symbol}"
            )
            
            return True
            
        except Exception as e:
            print(f"❌ Failed to add market event: {e}")
            return False
    
    async def query_correlations(self, symbol: str, timeframe: str = "24h") -> List[Dict]:
        """Query price correlations for a token"""
        if not self._initialized:
            return []
        
        try:
            query = f"Find tokens that are price-correlated with {symbol} in the last {timeframe}"
            
            results = await self.graphiti.search(
                query=query,
                limit=10
            )
            
            return [result.dict() for result in results]
            
        except Exception as e:
            print(f"❌ Failed to query correlations: {e}")
            return []
    
    async def query_strategy_performance(self, strategy_name: str) -> Dict:
        """Query historical performance of a strategy"""
        if not self._initialized:
            return {}
        
        try:
            query = f"What is the historical performance and success rate of {strategy_name} strategy?"
            
            results = await self.graphiti.search(
                query=query,
                limit=5
            )
            
            # Process results to extract performance metrics
            performance_data = {
                "strategy": strategy_name,
                "total_signals": 0,
                "successful_signals": 0,
                "win_rate": 0.0,
                "avg_strength": 0.0,
                "recent_signals": []
            }
            
            for result in results:
                # Extract performance metrics from search results
                # This would be customized based on actual data structure
                performance_data["recent_signals"].append(result.dict())
            
            return performance_data
            
        except Exception as e:
            print(f"❌ Failed to query strategy performance: {e}")
            return {}
    
    async def query_market_patterns(self, symbol: str, pattern_type: str) -> List[Dict]:
        """Query historical market patterns for a token"""
        if not self._initialized:
            return []
        
        try:
            query = f"Find historical {pattern_type} patterns for {symbol} and their outcomes"
            
            results = await self.graphiti.search(
                query=query,
                limit=20
            )
            
            return [result.dict() for result in results]
            
        except Exception as e:
            print(f"❌ Failed to query market patterns: {e}")
            return []
    
    async def get_ai_recommendation(self, symbol: str, current_conditions: Dict) -> Dict:
        """Get AI-powered trading recommendation based on knowledge graph"""
        if not self._initialized:
            return {}
        
        try:
            # Build context query
            context_query = f"""
            Based on historical data for {symbol}, current market conditions: {json.dumps(current_conditions)},
            and similar past scenarios, what trading action would you recommend?
            Consider: price correlations, strategy performance, market patterns, and risk factors.
            """
            
            results = await self.graphiti.search(
                query=context_query,
                limit=15
            )
            
            # Process results to generate recommendation
            recommendation = {
                "symbol": symbol,
                "action": "hold",  # buy, sell, hold
                "confidence": 0.0,
                "reasoning": [],
                "risk_factors": [],
                "supporting_evidence": []
            }
            
            # Analyze search results to build recommendation
            for result in results:
                recommendation["supporting_evidence"].append(result.dict())
            
            # TODO: Implement AI analysis logic based on search results
            # This would involve pattern matching, correlation analysis, etc.
            
            return recommendation
            
        except Exception as e:
            print(f"❌ Failed to get AI recommendation: {e}")
            return {}
    
    async def health_check(self) -> bool:
        """Check if knowledge graph is healthy"""
        if not self._initialized:
            return False
        
        try:
            # Simple query to test connection
            results = await self.graphiti.search(
                query="Test connection",
                limit=1
            )
            return True
            
        except Exception as e:
            print(f"❌ Knowledge graph health check failed: {e}")
            return False


# Python interface functions for Rust FFI
async def init_knowledge_graph(neo4j_uri: str, neo4j_user: str, neo4j_password: str,
                              openai_api_key: str) -> bool:
    """Initialize knowledge graph - called from Rust"""
    global kg_instance
    
    config = GraphitiConfig(
        neo4j_uri=neo4j_uri,
        neo4j_user=neo4j_user,
        neo4j_password=neo4j_password,
        openai_api_key=openai_api_key
    )
    
    kg_instance = SniperBotKnowledgeGraph(config)
    return await kg_instance.initialize()


async def add_token_data(symbol: str, price: float, volume_24h: float = None,
                        market_cap: Optional[float] = None) -> bool:
    """Add token data - called from Rust"""
    global kg_instance
    
    if kg_instance is None:
        return False
    
    return await kg_instance.add_token(
        symbol=symbol,
        price=price,
        volume_24h=volume_24h,
        market_cap=market_cap
    )


async def add_signal_data(strategy_name: str, symbol: str, strength: float,
                         signal_type: str, metadata: str = "{}") -> bool:
    """Add signal data - called from Rust"""
    global kg_instance
    
    if kg_instance is None:
        return False
    
    try:
        metadata_dict = json.loads(metadata)
    except:
        metadata_dict = {}
    
    return await kg_instance.add_signal(
        strategy_name=strategy_name,
        symbol=symbol,
        strength=strength,
        signal_type=signal_type,
        **metadata_dict
    )


async def get_ai_recommendation_json(symbol: str, conditions: str) -> str:
    """Get AI recommendation as JSON - called from Rust"""
    global kg_instance
    
    if kg_instance is None:
        return "{}"
    
    try:
        conditions_dict = json.loads(conditions)
        recommendation = await kg_instance.get_ai_recommendation(symbol, conditions_dict)
        return json.dumps(recommendation)
    except Exception as e:
        return json.dumps({"error": str(e)})


# Global instance
kg_instance: Optional[SniperBotKnowledgeGraph] = None


if __name__ == "__main__":
    # Test the knowledge graph
    async def test_kg():
        config = GraphitiConfig(
            openai_api_key=os.getenv("OPENAI_API_KEY")
        )
        
        kg = SniperBotKnowledgeGraph(config)
        
        if await kg.initialize():
            print("🧠 Testing Knowledge Graph...")
            
            # Add test data
            await kg.add_token("SOL", 160.0, volume_24h=1000000.0, market_cap=75000000000.0)
            await kg.add_signal("meteora_dlmm", "SOL", 0.85, "buy", confidence=0.9)
            await kg.add_market_event("price_spike", "SOL", 0.15, "binance")
            
            # Query data
            correlations = await kg.query_correlations("SOL")
            print(f"📊 Correlations: {len(correlations)} found")
            
            performance = await kg.query_strategy_performance("meteora_dlmm")
            print(f"📈 Strategy performance: {performance}")
            
            recommendation = await kg.get_ai_recommendation("SOL", {"price": 160.0, "volume": "high"})
            print(f"🤖 AI Recommendation: {recommendation}")
    
    asyncio.run(test_kg())

// OSTATECZNA, LOKALNA WERSJA api.ts
// W trybie deweloperskim, jawnie wskazujemy na nasze API na porcie 8084
const API_BASE_URL = 'http://localhost:8084';
// Lokalnie, używamy niezabezpieczonego WebSocketa
const WS_BASE_URL = 'ws://localhost:8084';

// Types for SniperBot API responses
export interface BotStatus {
  success: boolean;
  data: {
    engine_status: {
      is_running: boolean;
      portfolio_value: number;
    };
    active_strategies: string[];
    strategy_performance: Record<string, StrategyPerformance>;
  };
}

export interface StrategyPerformance {
  enabled: boolean;
  profit_24h: number;
  signals_generated: number;
  win_rate: number;
  total_pnl: number;
}

export interface Portfolio {
  wallet_address: string;
  network: string;
  sol_balance: number;
  sol_price_usd: number;
  total_usd_value: number;
  balance_status: string;
  active_positions: any[];
  trading_mode: string;
  last_updated: string;
}

export interface Signal {
  type: string;
  strategy: string;
  signal_type: 'Buy' | 'Sell';
  symbol: string;
  strength: number;
  timestamp: string;
  metadata?: any;
  price?: number;
  reason?: string;
}

export interface Trade {
  id: string;
  symbol: string;
  side: 'Buy' | 'Sell';
  size: number;
  price: number;
  status: 'Pending' | 'Filled' | 'Cancelled' | 'Failed';
  timestamp: string;
  strategy?: string;
}

export interface ReportEvent {
  event_type: string;
  timestamp: string;
  data: any;
}

export interface WebSocketMessage {
  type: string;
  message?: string;
  timestamp: string;
  data?: any;
}

// API Client Class
export class SniperBotAPI {
  private baseUrl: string;
  private wsUrl: string;

  constructor() {
    this.baseUrl = API_BASE_URL;
    this.wsUrl = WS_BASE_URL;
    console.log('🔧 API initialized - REST:', this.baseUrl, 'WS:', this.wsUrl);
  }

  // REST API Methods
  async getBotStatus(): Promise<BotStatus> {
    const response = await fetch(`${this.baseUrl}/api/bot-status`);
    if (!response.ok) {
      throw new Error(`Failed to fetch bot status: ${response.statusText}`);
    }
    return response.json();
  }

  async getPortfolio(): Promise<Portfolio> {
    const response = await fetch(`${this.baseUrl}/api/v1/portfolio`);
    if (!response.ok) {
      throw new Error(`Failed to fetch portfolio: ${response.statusText}`);
    }
    return response.json();
  }

  async getSignals(limit = 10): Promise<Signal[]> {
    console.log(`🚀 getSignals called with limit=${limit}, baseUrl=${this.baseUrl}`);
    const url = `${this.baseUrl}/api/events?limit=${limit}`;
    console.log(`📡 Fetching: ${url}`);

    const response = await fetch(url);
    console.log(`📡 Response status: ${response.status} ${response.statusText}`);

    if (!response.ok) {
      throw new Error(`Failed to fetch signals: ${response.statusText}`);
    }
    const data = await response.json();
    console.log('🔍 Raw API response:', data);

    // Backend returns array directly, not wrapped in {events: [...]}
    if (!Array.isArray(data)) {
      console.warn('⚠️ API response is not an array:', data);
      return [];
    }

    // Transform backend ReportEvent to frontend Signal format
    const signals = data
      .filter(event => event.type === 'SignalGenerated')
      .map((event, index) => ({
        id: `${event.timestamp}-${index}`, // Generate unique ID
        type: event.type,
        strategy: event.strategy,
        signal_type: event.signal_type,
        symbol: event.symbol,
        strength: event.strength,
        timestamp: event.timestamp,
        metadata: event.metadata,
        price: 100.0 // Mock price for now
      }));

    console.log(`✅ Transformed ${signals.length} signals from ${data.length} events`);
    return signals;
  }

  async getTrades(limit = 10): Promise<Trade[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/orders?limit=${limit}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch trades: ${response.statusText}`);
    }
    const data = await response.json();
    return data.orders || [];
  }

  async getEvents(limit = 10): Promise<ReportEvent[]> {
    const response = await fetch(`${this.baseUrl}/api/events?limit=${limit}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch events: ${response.statusText}`);
    }
    const data = await response.json();
    // Backend returns array directly, not wrapped in {events: [...]}
    return Array.isArray(data) ? data : [];
  }

  async startBot(): Promise<{ success: boolean; message: string }> {
    const response = await fetch(`${this.baseUrl}/api/bot/start`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Failed to start bot: ${response.statusText}`);
    }
    return response.json();
  }

  async stopBot(): Promise<{ success: boolean; message: string }> {
    const response = await fetch(`${this.baseUrl}/api/bot/stop`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Failed to stop bot: ${response.statusText}`);
    }
    return response.json();
  }

  async toggleStrategy(strategy: string): Promise<{ success: boolean; enabled: boolean }> {
    const response = await fetch(`${this.baseUrl}/api/strategy/${strategy}/toggle`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Failed to toggle strategy: ${response.statusText}`);
    }
    return response.json();
  }

  async resetStrategies(): Promise<{ success: boolean; message: string }> {
    const response = await fetch(`${this.baseUrl}/api/strategy/reset`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Failed to reset strategies: ${response.statusText}`);
    }
    return response.json();
  }

  // WebSocket Connection
  createWebSocket(onMessage: (message: WebSocketMessage) => void, onError?: (error: Event) => void): WebSocket {
    console.log('🔌 Creating WebSocket connection to:', `${this.wsUrl}/ws`);
    const ws = new WebSocket(`${this.wsUrl}/ws`);

    ws.onopen = () => {
      console.log('✅ WebSocket connected successfully to SniperBot');
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        console.log('📨 WebSocket message received:', message);
        onMessage(message);
      } catch (error) {
        console.error('❌ Failed to parse WebSocket message:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
      if (onError) onError(error);
    };

    ws.onclose = (event) => {
      console.log('🔌 WebSocket disconnected from SniperBot. Code:', event.code, 'Reason:', event.reason);
    };

    return ws;
  }
}

// Default API instance
const sniperBotAPI = new SniperBotAPI();
export { sniperBotAPI };
export default sniperBotAPI;

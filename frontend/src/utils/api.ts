// SniperBot API Configuration and Utilities
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8084';
const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8084';

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

export interface Signal {
  id: string;
  strategy: string;
  signal_type: 'Buy' | 'Sell';
  symbol: string;
  strength: number;
  timestamp: string;
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

  constructor(baseUrl = API_BASE_URL, wsUrl = WS_BASE_URL) {
    this.baseUrl = baseUrl;
    this.wsUrl = wsUrl;
  }

  // REST API Methods
  async getBotStatus(): Promise<BotStatus> {
    const response = await fetch(`${this.baseUrl}/api/bot-status`);
    if (!response.ok) {
      throw new Error(`Failed to fetch bot status: ${response.statusText}`);
    }
    return response.json();
  }

  async getSignals(limit = 10): Promise<Signal[]> {
    const response = await fetch(`${this.baseUrl}/api/signals?limit=${limit}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch signals: ${response.statusText}`);
    }
    const data = await response.json();
    return data.signals || [];
  }

  async getTrades(limit = 10): Promise<Trade[]> {
    const response = await fetch(`${this.baseUrl}/api/trades?limit=${limit}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch trades: ${response.statusText}`);
    }
    const data = await response.json();
    return data.trades || [];
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
    const ws = new WebSocket(`${this.wsUrl}/ws`);
    
    ws.onopen = () => {
      console.log('🔌 WebSocket connected to SniperBot');
    };
    
    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        onMessage(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      if (onError) onError(error);
    };
    
    ws.onclose = () => {
      console.log('🔌 WebSocket disconnected from SniperBot');
    };
    
    return ws;
  }
}

// Default API instance
export const sniperBotAPI = new SniperBotAPI();

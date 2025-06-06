import React, { createContext, useContext, useReducer, useEffect, useRef } from 'react';
import { sniperBotAPI, BotStatus, Signal, Trade, WebSocketMessage } from '../utils/api';

// State interface
interface SniperBotState {
  // Connection status
  isConnected: boolean;
  wsConnected: boolean;
  
  // Bot status
  botStatus: BotStatus | null;
  isLoading: boolean;
  error: string | null;
  
  // Real-time data
  signals: Signal[];
  trades: Trade[];
  
  // UI state
  selectedStrategy: string | null;
}

// Action types
type SniperBotAction =
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_BOT_STATUS'; payload: BotStatus }
  | { type: 'SET_SIGNALS'; payload: Signal[] }
  | { type: 'ADD_SIGNAL'; payload: Signal }
  | { type: 'SET_TRADES'; payload: Trade[] }
  | { type: 'ADD_TRADE'; payload: Trade }
  | { type: 'SET_WS_CONNECTED'; payload: boolean }
  | { type: 'SET_SELECTED_STRATEGY'; payload: string | null };

// Initial state
const initialState: SniperBotState = {
  isConnected: false,
  wsConnected: false,
  botStatus: null,
  isLoading: false,
  error: null,
  signals: [],
  trades: [],
  selectedStrategy: null,
};

// Reducer
function sniperBotReducer(state: SniperBotState, action: SniperBotAction): SniperBotState {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, isLoading: false };
    case 'SET_BOT_STATUS':
      return { ...state, botStatus: action.payload, isConnected: true, isLoading: false };
    case 'SET_SIGNALS':
      return { ...state, signals: action.payload };
    case 'ADD_SIGNAL':
      return { 
        ...state, 
        signals: [action.payload, ...state.signals].slice(0, 50) // Keep last 50 signals
      };
    case 'SET_TRADES':
      return { ...state, trades: action.payload };
    case 'ADD_TRADE':
      return { 
        ...state, 
        trades: [action.payload, ...state.trades].slice(0, 50) // Keep last 50 trades
      };
    case 'SET_WS_CONNECTED':
      return { ...state, wsConnected: action.payload };
    case 'SET_SELECTED_STRATEGY':
      return { ...state, selectedStrategy: action.payload };
    default:
      return state;
  }
}

// Context interface
interface SniperBotContextType {
  state: SniperBotState;
  actions: {
    refreshBotStatus: () => Promise<void>;
    refreshSignals: () => Promise<void>;
    refreshTrades: () => Promise<void>;
    startBot: () => Promise<void>;
    stopBot: () => Promise<void>;
    toggleStrategy: (strategy: string) => Promise<void>;
    resetStrategies: () => Promise<void>;
    setSelectedStrategy: (strategy: string | null) => void;
  };
}

// Create context
const SniperBotContext = createContext<SniperBotContextType | undefined>(undefined);

// Provider component
export const SniperBotProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(sniperBotReducer, initialState);
  const wsRef = useRef<WebSocket | null>(null);

  // WebSocket message handler
  const handleWebSocketMessage = (message: WebSocketMessage) => {
    console.log('📨 WebSocket message:', message);
    
    switch (message.type) {
      case 'SignalGenerated':
        if (message.data) {
          dispatch({ type: 'ADD_SIGNAL', payload: message.data });
        }
        break;
      case 'TradeExecuted':
        if (message.data) {
          dispatch({ type: 'ADD_TRADE', payload: message.data });
        }
        break;
      case 'connected':
        dispatch({ type: 'SET_WS_CONNECTED', payload: true });
        break;
      default:
        console.log('Unknown WebSocket message type:', message.type);
    }
  };

  // Initialize WebSocket connection
  useEffect(() => {
    const connectWebSocket = () => {
      try {
        wsRef.current = sniperBotAPI.createWebSocket(
          handleWebSocketMessage,
          (error) => {
            console.error('WebSocket error:', error);
            dispatch({ type: 'SET_WS_CONNECTED', payload: false });
            // Attempt to reconnect after 5 seconds
            setTimeout(connectWebSocket, 5000);
          }
        );
      } catch (error) {
        console.error('Failed to create WebSocket:', error);
        setTimeout(connectWebSocket, 5000);
      }
    };

    connectWebSocket();

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  // Actions
  const actions = {
    refreshBotStatus: async () => {
      dispatch({ type: 'SET_LOADING', payload: true });
      try {
        const status = await sniperBotAPI.getBotStatus();
        dispatch({ type: 'SET_BOT_STATUS', payload: status });
        dispatch({ type: 'SET_ERROR', payload: null });
      } catch (error) {
        dispatch({ type: 'SET_ERROR', payload: error instanceof Error ? error.message : 'Unknown error' });
      }
    },

    refreshSignals: async () => {
      try {
        console.log('🔄 Refreshing signals...');
        const signals = await sniperBotAPI.getSignals(20);
        console.log('📊 Received signals:', signals.length);
        dispatch({ type: 'SET_SIGNALS', payload: signals });
        console.log('✅ Signals updated in state');
      } catch (error) {
        console.error('❌ Failed to refresh signals:', error);
      }
    },

    refreshTrades: async () => {
      try {
        const trades = await sniperBotAPI.getTrades(20);
        dispatch({ type: 'SET_TRADES', payload: trades });
      } catch (error) {
        console.error('Failed to refresh trades:', error);
      }
    },

    startBot: async () => {
      try {
        await sniperBotAPI.startBot();
        await actions.refreshBotStatus();
      } catch (error) {
        dispatch({ type: 'SET_ERROR', payload: error instanceof Error ? error.message : 'Failed to start bot' });
      }
    },

    stopBot: async () => {
      try {
        await sniperBotAPI.stopBot();
        await actions.refreshBotStatus();
      } catch (error) {
        dispatch({ type: 'SET_ERROR', payload: error instanceof Error ? error.message : 'Failed to stop bot' });
      }
    },

    toggleStrategy: async (strategy: string) => {
      try {
        await sniperBotAPI.toggleStrategy(strategy);
        await actions.refreshBotStatus();
      } catch (error) {
        dispatch({ type: 'SET_ERROR', payload: error instanceof Error ? error.message : 'Failed to toggle strategy' });
      }
    },

    resetStrategies: async () => {
      try {
        await sniperBotAPI.resetStrategies();
        await actions.refreshBotStatus();
      } catch (error) {
        dispatch({ type: 'SET_ERROR', payload: error instanceof Error ? error.message : 'Failed to reset strategies' });
      }
    },

    setSelectedStrategy: (strategy: string | null) => {
      dispatch({ type: 'SET_SELECTED_STRATEGY', payload: strategy });
    },
  };

  // Initial data load
  useEffect(() => {
    console.log('🚀 SniperBotContext initializing...');
    console.log('🤖 Fetching initial bot status...');
    actions.refreshBotStatus();
    console.log('📈 Fetching initial signals...');
    actions.refreshSignals();
    console.log('💰 Fetching initial trades...');
    actions.refreshTrades();
    console.log('✅ SniperBotContext initialized');
  }, []);

  // Periodic refresh
  useEffect(() => {
    const interval = setInterval(() => {
      actions.refreshBotStatus();
    }, 30000); // Refresh every 30 seconds

    return () => clearInterval(interval);
  }, []);

  return (
    <SniperBotContext.Provider value={{ state, actions }}>
      {children}
    </SniperBotContext.Provider>
  );
};

// Hook to use the context
export const useSniperBot = () => {
  const context = useContext(SniperBotContext);
  if (context === undefined) {
    throw new Error('useSniperBot must be used within a SniperBotProvider');
  }
  return context;
};

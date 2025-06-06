import React, { useEffect, useRef } from 'react';
import { TrendingUp, TrendingDown, Clock, Target, Zap } from 'lucide-react';
import { useSniperBot } from '../../context/SniperBotContext';

const SignalsDashboard: React.FC = () => {
  const { state, actions } = useSniperBot();
  const { signals, selectedStrategy } = state;
  const signalsEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to latest signal
  useEffect(() => {
    if (signalsEndRef.current) {
      signalsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [signals]);

  // Filter signals by selected strategy
  const filteredSignals = selectedStrategy
    ? signals.filter(signal => signal.strategy === selectedStrategy)
    : signals;

  // Debug logging
  console.log('🔍 SignalsDashboard render:', {
    totalSignals: signals.length,
    filteredSignals: filteredSignals.length,
    selectedStrategy,
    firstSignal: signals[0],
    strategies: [...new Set(signals.map(s => s.strategy))]
  });

  const getSignalIcon = (signalType: string) => {
    switch (signalType) {
      case 'Buy':
        return <TrendingUp className="h-4 w-4 text-trading-bull-primary" />;
      case 'Sell':
        return <TrendingDown className="h-4 w-4 text-trading-bear-primary" />;
      default:
        return <Target className="h-4 w-4 text-status-info" />;
    }
  };

  const getSignalColor = (signalType: string) => {
    switch (signalType) {
      case 'Buy':
        return 'border-l-trading-bull-primary bg-trading-bull-light';
      case 'Sell':
        return 'border-l-trading-bear-primary bg-trading-bear-light';
      default:
        return 'border-l-status-info bg-finance-50';
    }
  };

  const getStrengthColor = (strength: number) => {
    if (strength >= 0.8) return 'text-white bg-signal-strong';
    if (strength >= 0.6) return 'text-white bg-signal-medium';
    if (strength >= 0.4) return 'text-white bg-signal-weak';
    return 'text-white bg-signal-very-weak';
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString();
  };

  const formatStrategy = (strategy: string) => {
    return strategy.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow-trading border border-finance-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-finance-900">Real-time Signals</h2>
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <Zap className="h-4 w-4 text-status-warning" />
              <span className="text-sm text-finance-600">
                {filteredSignals.length} signals
              </span>
            </div>
            <button
              onClick={actions.refreshSignals}
              className="px-3 py-1 bg-status-info text-white rounded text-sm hover:bg-blue-700 transition-colors shadow-trading"
            >
              Refresh
            </button>
            <button
              onClick={() => {
                console.log('🧪 Testing API directly...');
                fetch('http://localhost:8084/api/events?limit=3')
                  .then(r => {
                    console.log('📡 Response status:', r.status);
                    return r.json();
                  })
                  .then(d => {
                    console.log('✅ Direct API test successful:', d.length, 'events');
                    console.log('📊 First event:', d[0]);
                  })
                  .catch(e => console.error('❌ Direct API test failed:', e));
              }}
              className="px-3 py-1 bg-green-600 text-white rounded text-sm hover:bg-green-700 transition-colors shadow-trading"
            >
              Test API
            </button>
          </div>
        </div>

        {/* Strategy Filter */}
        <div className="flex items-center space-x-2 mb-4">
          <span className="text-sm text-finance-600">Filter by strategy:</span>
          <select
            value={selectedStrategy || ''}
            onChange={(e) => actions.setSelectedStrategy(e.target.value || null)}
            className="px-3 py-1 border border-finance-300 rounded text-sm focus:outline-none focus:ring-2 focus:ring-status-info bg-white text-finance-900"
          >
            <option value="">All Strategies</option>
            <option value="pumpfun_sniping">PumpFun Sniping</option>
            <option value="liquidity_sniping">Liquidity Sniping</option>
            <option value="meteora_dlmm">Meteora DLMM</option>
            <option value="arbitrage">Arbitrage</option>
          </select>
        </div>

        {/* Statistics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-finance-50 rounded-lg p-3 border border-finance-100">
            <div className="flex items-center space-x-2">
              <TrendingUp className="h-4 w-4 text-trading-bull-primary" />
              <span className="text-sm text-finance-600">Buy Signals</span>
            </div>
            <p className="text-lg font-semibold text-trading-bull-primary mt-1">
              {filteredSignals.filter(s => s.signal_type === 'Buy').length}
            </p>
          </div>

          <div className="bg-finance-50 rounded-lg p-3 border border-finance-100">
            <div className="flex items-center space-x-2">
              <TrendingDown className="h-4 w-4 text-trading-bear-primary" />
              <span className="text-sm text-finance-600">Sell Signals</span>
            </div>
            <p className="text-lg font-semibold text-trading-bear-primary mt-1">
              {filteredSignals.filter(s => s.signal_type === 'Sell').length}
            </p>
          </div>

          <div className="bg-finance-50 rounded-lg p-3 border border-finance-100">
            <div className="flex items-center space-x-2">
              <Target className="h-4 w-4 text-status-info" />
              <span className="text-sm text-finance-600">Avg Strength</span>
            </div>
            <p className="text-lg font-semibold text-status-info mt-1">
              {filteredSignals.length > 0
                ? (filteredSignals.reduce((sum, s) => sum + s.strength, 0) / filteredSignals.length).toFixed(2)
                : '0.00'
              }
            </p>
          </div>

          <div className="bg-finance-50 rounded-lg p-3 border border-finance-100">
            <div className="flex items-center space-x-2">
              <Clock className="h-4 w-4 text-status-warning" />
              <span className="text-sm text-finance-600">Last Signal</span>
            </div>
            <p className="text-lg font-semibold text-finance-900 mt-1">
              {filteredSignals.length > 0
                ? formatTimestamp(filteredSignals[0].timestamp)
                : 'None'
              }
            </p>
          </div>
        </div>
      </div>

      {/* Signals List */}
      <div className="bg-white rounded-lg shadow-trading border border-finance-200">
        <div className="p-4 border-b border-finance-200">
          <h3 className="text-lg font-semibold text-finance-900">Signal Feed</h3>
        </div>

        <div className="max-h-96 overflow-y-auto">
          {filteredSignals.length === 0 ? (
            <div className="p-8 text-center text-finance-500">
              <Zap className="h-8 w-8 mx-auto mb-2 text-finance-400" />
              <p>No signals yet</p>
              <p className="text-sm">Signals will appear here in real-time</p>
            </div>
          ) : (
            <div className="space-y-2 p-4">
              {filteredSignals.map((signal, index) => (
                <div
                  key={signal.id || index}
                  className={`border-l-4 rounded-lg p-4 transition-all duration-200 ${getSignalColor(signal.signal_type)}`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-3">
                      {getSignalIcon(signal.signal_type)}
                      <span className="font-medium text-finance-900">
                        {signal.symbol || 'Unknown'}
                      </span>
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStrengthColor(signal.strength)}`}>
                        {(signal.strength * 100).toFixed(0)}%
                      </span>
                    </div>
                    <div className="text-right">
                      <p className="text-sm text-finance-600">
                        {formatTimestamp(signal.timestamp)}
                      </p>
                      <p className="text-xs text-finance-500">
                        {formatStrategy(signal.strategy)}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center justify-between text-sm">
                    <div className="flex items-center space-x-4">
                      <span className={`font-medium ${
                        signal.signal_type === 'Buy' ? 'text-trading-bull-primary' : 'text-trading-bear-primary'
                      }`}>
                        {signal.signal_type}
                      </span>
                      {signal.price && (
                        <span className="text-finance-600">
                          ${signal.price.toFixed(6)}
                        </span>
                      )}
                    </div>
                    {signal.reason && (
                      <span className="text-finance-500 text-xs max-w-xs truncate">
                        {signal.reason}
                      </span>
                    )}
                  </div>
                </div>
              ))}
              <div ref={signalsEndRef} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default SignalsDashboard;

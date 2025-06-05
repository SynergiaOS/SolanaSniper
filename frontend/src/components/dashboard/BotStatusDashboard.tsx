import React from 'react';
import { Activity, DollarSign, TrendingUp, Zap, Play, Square, RotateCcw } from 'lucide-react';
import { useSniperBot } from '../../context/SniperBotContext';

const BotStatusDashboard: React.FC = () => {
  const { state, actions } = useSniperBot();
  const { botStatus, isLoading, error, wsConnected } = state;

  const handleStartBot = () => actions.startBot();
  const handleStopBot = () => actions.stopBot();
  const handleResetStrategies = () => actions.resetStrategies();

  if (isLoading && !botStatus) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-status-info"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-trading-bear-light border border-trading-bear-primary rounded-lg p-4">
        <p className="text-trading-bear-dark font-medium">Error: {error}</p>
        <button
          onClick={actions.refreshBotStatus}
          className="mt-2 px-4 py-2 bg-trading-bear-primary text-white rounded hover:bg-trading-bear-dark transition-colors"
        >
          Retry
        </button>
      </div>
    );
  }

  const isRunning = botStatus?.data?.engine_status?.is_running || false;
  const portfolioValue = botStatus?.data?.engine_status?.portfolio_value || 0;
  const activeStrategies = botStatus?.data?.active_strategies || [];
  const strategyPerformance = botStatus?.data?.strategy_performance || {};

  return (
    <div className="space-y-6">
      {/* Header with Bot Controls */}
      <div className="bg-white rounded-lg shadow-trading border border-finance-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-finance-900">SniperBot Status</h2>
          <div className="flex items-center space-x-2">
            <div className={`flex items-center space-x-2 px-3 py-1 rounded-full text-sm font-medium ${
              wsConnected ? 'bg-trading-bull-light text-trading-bull-dark' : 'bg-trading-bear-light text-trading-bear-dark'
            }`}>
              <div className={`w-2 h-2 rounded-full ${wsConnected ? 'bg-status-active' : 'bg-status-error'}`}></div>
              <span>{wsConnected ? 'Connected' : 'Disconnected'}</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          {/* Bot Status */}
          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <Activity className={`h-5 w-5 ${isRunning ? 'text-status-active' : 'text-status-inactive'}`} />
              <span className="text-sm font-medium text-finance-600">Status</span>
            </div>
            <p className={`text-lg font-semibold mt-1 ${isRunning ? 'text-status-active' : 'text-status-inactive'}`}>
              {isRunning ? 'Running' : 'Stopped'}
            </p>
          </div>

          {/* Portfolio Value */}
          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <DollarSign className="h-5 w-5 text-status-info" />
              <span className="text-sm font-medium text-finance-600">Portfolio</span>
            </div>
            <p className="text-lg font-semibold text-finance-900 mt-1">
              ${portfolioValue.toFixed(2)}
            </p>
          </div>

          {/* Active Strategies */}
          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <Zap className="h-5 w-5 text-status-warning" />
              <span className="text-sm font-medium text-finance-600">Strategies</span>
            </div>
            <p className="text-lg font-semibold text-finance-900 mt-1">
              {activeStrategies.length}
            </p>
          </div>

          {/* Total P&L */}
          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <TrendingUp className="h-5 w-5 text-trading-bull-primary" />
              <span className="text-sm font-medium text-finance-600">Total P&L</span>
            </div>
            <p className={`text-lg font-semibold mt-1 ${
              Object.values(strategyPerformance).reduce((sum, perf) => sum + (perf.total_pnl || 0), 0) >= 0
                ? 'text-trading-bull-primary'
                : 'text-trading-bear-primary'
            }`}>
              ${Object.values(strategyPerformance).reduce((sum, perf) => sum + (perf.total_pnl || 0), 0).toFixed(2)}
            </p>
          </div>
        </div>

        {/* Control Buttons */}
        <div className="flex space-x-3">
          <button
            onClick={isRunning ? handleStopBot : handleStartBot}
            disabled={isLoading}
            className={`flex items-center space-x-2 px-4 py-2 rounded-lg font-medium transition-colors ${
              isRunning
                ? 'bg-trading-bear-primary hover:bg-trading-bear-dark text-white shadow-trading'
                : 'bg-trading-bull-primary hover:bg-trading-bull-dark text-white shadow-trading'
            } disabled:opacity-50 disabled:cursor-not-allowed`}
          >
            {isRunning ? <Square className="h-4 w-4" /> : <Play className="h-4 w-4" />}
            <span>{isRunning ? 'Stop Bot' : 'Start Bot'}</span>
          </button>

          <button
            onClick={handleResetStrategies}
            disabled={isLoading}
            className="flex items-center space-x-2 px-4 py-2 bg-finance-600 hover:bg-finance-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-trading"
          >
            <RotateCcw className="h-4 w-4" />
            <span>Reset Strategies</span>
          </button>
        </div>
      </div>

      {/* Strategy Performance */}
      <div className="bg-white rounded-lg shadow-trading border border-finance-200 p-6">
        <h3 className="text-lg font-semibold text-finance-900 mb-4">Strategy Performance</h3>
        <div className="space-y-4">
          {Object.entries(strategyPerformance).map(([strategyName, perf]) => (
            <div key={strategyName} className="border border-finance-200 rounded-lg p-4 hover:border-finance-300 transition-colors">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center space-x-3">
                  <h4 className="font-medium text-finance-900 capitalize">
                    {strategyName.replace(/_/g, ' ')}
                  </h4>
                  <button
                    onClick={() => actions.toggleStrategy(strategyName)}
                    className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
                      perf.enabled
                        ? 'bg-trading-bull-light text-trading-bull-dark hover:bg-trading-bull-primary hover:text-white'
                        : 'bg-trading-bear-light text-trading-bear-dark hover:bg-trading-bear-primary hover:text-white'
                    }`}
                  >
                    {perf.enabled ? 'Enabled' : 'Disabled'}
                  </button>
                </div>
                <div className="text-right">
                  <p className={`text-sm font-medium ${perf.total_pnl >= 0 ? 'text-trading-bull-primary' : 'text-trading-bear-primary'}`}>
                    ${perf.total_pnl?.toFixed(2) || '0.00'}
                  </p>
                  <p className="text-xs text-finance-500">Total P&L</p>
                </div>
              </div>
              
              <div className="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <p className="text-finance-600">Signals</p>
                  <p className="font-medium text-finance-900">{perf.signals_generated || 0}</p>
                </div>
                <div>
                  <p className="text-finance-600">Win Rate</p>
                  <p className="font-medium text-finance-900">{((perf.win_rate || 0) * 100).toFixed(1)}%</p>
                </div>
                <div>
                  <p className="text-finance-600">24h Profit</p>
                  <p className={`font-medium ${perf.profit_24h >= 0 ? 'text-trading-bull-primary' : 'text-trading-bear-primary'}`}>
                    ${perf.profit_24h?.toFixed(2) || '0.00'}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default BotStatusDashboard;

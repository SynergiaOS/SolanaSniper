import React from 'react';
import { DollarSign, TrendingUp, TrendingDown, Clock, CheckCircle, XCircle, AlertCircle } from 'lucide-react';
import { useSniperBot } from '../../context/SniperBotContext';

const TradesDashboard: React.FC = () => {
  const { state, actions } = useSniperBot();
  const { trades } = state;

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'Filled':
        return <CheckCircle className="h-4 w-4 text-status-active" />;
      case 'Failed':
      case 'Cancelled':
        return <XCircle className="h-4 w-4 text-status-error" />;
      case 'Pending':
        return <AlertCircle className="h-4 w-4 text-status-warning" />;
      default:
        return <Clock className="h-4 w-4 text-status-inactive" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Filled':
        return 'bg-trading-bull-light text-trading-bull-dark';
      case 'Failed':
      case 'Cancelled':
        return 'bg-trading-bear-light text-trading-bear-dark';
      case 'Pending':
        return 'bg-yellow-100 text-yellow-800';
      default:
        return 'bg-finance-100 text-finance-800';
    }
  };

  const getSideIcon = (side: string) => {
    return side === 'Buy'
      ? <TrendingUp className="h-4 w-4 text-trading-bull-primary" />
      : <TrendingDown className="h-4 w-4 text-trading-bear-primary" />;
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleString();
  };

  const formatStrategy = (strategy?: string) => {
    if (!strategy) return 'Manual';
    return strategy.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  };

  // Calculate statistics
  const totalTrades = trades.length;
  const filledTrades = trades.filter(t => t.status === 'Filled');
  const totalVolume = filledTrades.reduce((sum, t) => sum + (t.size * t.price), 0);
  const buyTrades = filledTrades.filter(t => t.side === 'Buy');
  const sellTrades = filledTrades.filter(t => t.side === 'Sell');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow-trading border border-finance-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-finance-900">Trading History</h2>
          <button
            onClick={actions.refreshTrades}
            className="px-3 py-1 bg-status-info text-white rounded text-sm hover:bg-blue-700 transition-colors shadow-trading"
          >
            Refresh
          </button>
        </div>

        {/* Statistics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <DollarSign className="h-5 w-5 text-status-info" />
              <span className="text-sm font-medium text-finance-600">Total Trades</span>
            </div>
            <p className="text-lg font-semibold text-finance-900 mt-1">
              {totalTrades}
            </p>
          </div>

          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <TrendingUp className="h-5 w-5 text-trading-bull-primary" />
              <span className="text-sm font-medium text-finance-600">Buy Orders</span>
            </div>
            <p className="text-lg font-semibold text-trading-bull-primary mt-1">
              {buyTrades.length}
            </p>
          </div>

          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <TrendingDown className="h-5 w-5 text-trading-bear-primary" />
              <span className="text-sm font-medium text-finance-600">Sell Orders</span>
            </div>
            <p className="text-lg font-semibold text-trading-bear-primary mt-1">
              {sellTrades.length}
            </p>
          </div>

          <div className="bg-finance-50 rounded-lg p-4 border border-finance-100">
            <div className="flex items-center space-x-2">
              <DollarSign className="h-5 w-5 text-status-warning" />
              <span className="text-sm font-medium text-finance-600">Volume</span>
            </div>
            <p className="text-lg font-semibold text-finance-900 mt-1">
              ${totalVolume.toFixed(2)}
            </p>
          </div>
        </div>
      </div>

      {/* Trades Table */}
      <div className="bg-white rounded-lg shadow-trading border border-finance-200">
        <div className="p-4 border-b border-finance-200">
          <h3 className="text-lg font-semibold text-finance-900">Recent Trades</h3>
        </div>

        <div className="overflow-x-auto">
          {trades.length === 0 ? (
            <div className="p-8 text-center text-finance-500">
              <DollarSign className="h-8 w-8 mx-auto mb-2 text-finance-400" />
              <p>No trades yet</p>
              <p className="text-sm">Trade history will appear here</p>
            </div>
          ) : (
            <table className="w-full">
              <thead className="bg-finance-50">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Time
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Symbol
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Side
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Size
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Price
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Value
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-finance-500 uppercase tracking-wider">
                    Strategy
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-finance-200">
                {trades.map((trade, index) => (
                  <tr key={trade.id || index} className="hover:bg-finance-50 transition-colors">
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-finance-900">
                      {formatTimestamp(trade.timestamp)}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-finance-900">
                      {trade.symbol}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm">
                      <div className="flex items-center space-x-2">
                        {getSideIcon(trade.side)}
                        <span className={trade.side === 'Buy' ? 'text-trading-bull-primary' : 'text-trading-bear-primary'}>
                          {trade.side}
                        </span>
                      </div>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-finance-900">
                      {trade.size.toFixed(6)}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-finance-900">
                      ${trade.price.toFixed(6)}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-finance-900">
                      ${(trade.size * trade.price).toFixed(2)}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm">
                      <div className="flex items-center space-x-2">
                        {getStatusIcon(trade.status)}
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(trade.status)}`}>
                          {trade.status}
                        </span>
                      </div>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-finance-600">
                      {formatStrategy(trade.strategy)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
};

export default TradesDashboard;

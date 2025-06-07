import React, { useState, useEffect } from 'react';
import { AlertTriangle, TrendingUp, TrendingDown, X, Shield } from 'lucide-react';

interface ActivePosition {
  id: string;
  token_address: string;
  symbol: string;
  strategy: string;
  side: string;
  size: number;
  entry_price: number;
  current_price: number;
  unrealized_pnl: number;
  unrealized_pnl_percent: number;
  opened_at: string;
  last_updated: string;
  stop_loss?: number;
  take_profit?: number;
  risk_score: number;
}

interface ClosePositionRequest {
  reason?: string;
  force?: boolean;
}

const ActivePositions: React.FC = () => {
  const [positions, setPositions] = useState<ActivePosition[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [closingPosition, setClosingPosition] = useState<string | null>(null);

  const fetchPositions = async () => {
    try {
      const response = await fetch('http://localhost:8084/api/positions');
      if (!response.ok) {
        throw new Error(`Failed to fetch positions: ${response.statusText}`);
      }
      const data = await response.json();
      setPositions(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const closePosition = async (positionId: string, reason?: string) => {
    if (!confirm(`Are you sure you want to manually close position ${positionId}?`)) {
      return;
    }

    setClosingPosition(positionId);
    try {
      const request: ClosePositionRequest = {
        reason: reason || 'Manual close from dashboard',
        force: false
      };

      const response = await fetch(`http://localhost:8084/api/positions/${positionId}/close`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        throw new Error(`Failed to close position: ${response.statusText}`);
      }

      const result = await response.json();
      alert(`Position close request submitted: ${result.message}`);
      
      // Refresh positions
      await fetchPositions();
    } catch (err) {
      alert(`Error closing position: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setClosingPosition(null);
    }
  };

  const emergencyCloseAll = async () => {
    if (!confirm('🚨 EMERGENCY CLOSE ALL POSITIONS? This action cannot be undone!')) {
      return;
    }
    
    if (!confirm('🚨 Are you ABSOLUTELY SURE? This will close ALL active positions immediately!')) {
      return;
    }

    try {
      const response = await fetch('http://localhost:8084/api/positions/emergency-close-all', {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Emergency close failed: ${response.statusText}`);
      }

      const result = await response.json();
      alert(`🚨 Emergency close initiated: ${result.message}`);
      
      // Refresh positions
      await fetchPositions();
    } catch (err) {
      alert(`🚨 Emergency close error: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  useEffect(() => {
    fetchPositions();
    const interval = setInterval(fetchPositions, 5000); // Refresh every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 4,
    }).format(value);
  };

  const formatPercent = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
  };

  const getRiskColor = (riskScore: number) => {
    if (riskScore <= 0.3) return 'text-green-400';
    if (riskScore <= 0.6) return 'text-yellow-400';
    return 'text-red-400';
  };

  const getPnlColor = (pnl: number) => {
    return pnl >= 0 ? 'text-green-400' : 'text-red-400';
  };

  if (loading) {
    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">Active Positions</h2>
        <div className="text-gray-400">Loading positions...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">Active Positions</h2>
        <div className="text-red-400">Error: {error}</div>
        <button 
          onClick={fetchPositions}
          className="mt-2 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-bold text-white">
          Active Positions ({positions.length})
        </h2>
        <div className="flex gap-2">
          <button
            onClick={fetchPositions}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            Refresh
          </button>
          {positions.length > 0 && (
            <button
              onClick={emergencyCloseAll}
              className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition-colors flex items-center gap-2"
            >
              <AlertTriangle size={16} />
              Emergency Close All
            </button>
          )}
        </div>
      </div>

      {positions.length === 0 ? (
        <div className="text-gray-400 text-center py-8">
          No active positions
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left text-gray-300 pb-2">Token</th>
                <th className="text-left text-gray-300 pb-2">Strategy</th>
                <th className="text-right text-gray-300 pb-2">Size</th>
                <th className="text-right text-gray-300 pb-2">Entry Price</th>
                <th className="text-right text-gray-300 pb-2">Current Price</th>
                <th className="text-right text-gray-300 pb-2">P&L</th>
                <th className="text-center text-gray-300 pb-2">Risk</th>
                <th className="text-center text-gray-300 pb-2">Actions</th>
              </tr>
            </thead>
            <tbody>
              {positions.map((position) => (
                <tr key={position.id} className="border-b border-gray-700 hover:bg-gray-750">
                  <td className="py-3">
                    <div>
                      <div className="text-white font-medium">{position.symbol}</div>
                      <div className="text-gray-400 text-xs">{position.side}</div>
                    </div>
                  </td>
                  <td className="py-3">
                    <span className="text-blue-400">{position.strategy}</span>
                  </td>
                  <td className="py-3 text-right">
                    <span className="text-white">{position.size.toLocaleString()}</span>
                  </td>
                  <td className="py-3 text-right">
                    <span className="text-gray-300">{formatCurrency(position.entry_price)}</span>
                  </td>
                  <td className="py-3 text-right">
                    <div className="flex items-center justify-end gap-1">
                      {position.current_price > position.entry_price ? (
                        <TrendingUp size={14} className="text-green-400" />
                      ) : (
                        <TrendingDown size={14} className="text-red-400" />
                      )}
                      <span className="text-white">{formatCurrency(position.current_price)}</span>
                    </div>
                  </td>
                  <td className="py-3 text-right">
                    <div>
                      <div className={`font-medium ${getPnlColor(position.unrealized_pnl)}`}>
                        {formatCurrency(position.unrealized_pnl)}
                      </div>
                      <div className={`text-xs ${getPnlColor(position.unrealized_pnl_percent)}`}>
                        {formatPercent(position.unrealized_pnl_percent)}
                      </div>
                    </div>
                  </td>
                  <td className="py-3 text-center">
                    <div className="flex items-center justify-center gap-1">
                      <Shield size={14} className={getRiskColor(position.risk_score)} />
                      <span className={getRiskColor(position.risk_score)}>
                        {(position.risk_score * 100).toFixed(0)}%
                      </span>
                    </div>
                  </td>
                  <td className="py-3 text-center">
                    <button
                      onClick={() => closePosition(position.id)}
                      disabled={closingPosition === position.id}
                      className="px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700 transition-colors disabled:opacity-50 flex items-center gap-1 mx-auto"
                    >
                      <X size={12} />
                      {closingPosition === position.id ? 'Closing...' : 'Close'}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default ActivePositions;

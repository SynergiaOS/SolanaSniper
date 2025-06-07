import React, { useState, useEffect } from 'react';
import { 
  Wallet, 
  TrendingUp, 
  TrendingDown, 
  RefreshCw, 
  DollarSign,
  Activity,
  AlertCircle,
  CheckCircle,
  Clock,
  Copy
} from 'lucide-react';

interface PortfolioData {
  wallet_address: string;
  network: string;
  sol_balance: number;
  sol_price_usd: number;
  total_usd_value: number;
  balance_status: string;
  active_positions_count: number;
  trading_mode: string;
  last_updated: string;
  token_balances?: TokenBalance[];
}

interface TokenBalance {
  mint_address: string;
  symbol: string;
  balance: number;
  decimals: number;
  usd_value?: number;
  price_per_token?: number;
}

const PortfolioOverview: React.FC = () => {
  const [portfolioData, setPortfolioData] = useState<PortfolioData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date());

  const fetchPortfolioData = async () => {
    try {
      setLoading(true);
      
      // Use correct API endpoint
      const response = await fetch('http://localhost:8084/api/v1/portfolio');
      if (!response.ok) {
        throw new Error(`API Error: ${response.statusText}`);
      }
      
      const data = await response.json();
      setPortfolioData(data);
      setLastRefresh(new Date());
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchPortfolioData();
    const interval = setInterval(fetchPortfolioData, 10000); // Refresh every 10 seconds
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

  const formatSOL = (value: number) => {
    return `${value.toFixed(6)} SOL`;
  };

  const getNetworkColor = (network: string) => {
    if (network.includes('mainnet')) return 'text-green-400';
    if (network.includes('devnet')) return 'text-yellow-400';
    if (network.includes('testnet')) return 'text-blue-400';
    return 'text-gray-400';
  };

  const getStatusIcon = (status: string) => {
    if (status.includes('✅')) return <CheckCircle size={16} className="text-green-400" />;
    if (status.includes('❌')) return <AlertCircle size={16} className="text-red-400" />;
    return <Clock size={16} className="text-yellow-400" />;
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  if (loading && !portfolioData) {
    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">Portfolio Overview</h2>
        <div className="text-gray-400">Loading portfolio data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">Portfolio Overview</h2>
        <div className="text-red-400 mb-4">Error: {error}</div>
        <button 
          onClick={fetchPortfolioData}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!portfolioData) return null;

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-bold text-white flex items-center gap-2">
          <Wallet size={20} />
          Portfolio Overview
        </h2>
        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-400">
            Last updated: {lastRefresh.toLocaleTimeString()}
          </span>
          <button
            onClick={fetchPortfolioData}
            disabled={loading}
            className="p-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            <RefreshCw size={16} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>
      </div>

      {/* Portfolio Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">Network</span>
            {getStatusIcon(portfolioData.balance_status)}
          </div>
          <div className={`font-bold ${getNetworkColor(portfolioData.network)}`}>
            {portfolioData.network.includes('mainnet') ? 'MAINNET' : 
             portfolioData.network.includes('devnet') ? 'DEVNET' : 'TESTNET'}
          </div>
          <div className="text-xs text-gray-400 mt-1">
            {portfolioData.balance_status}
          </div>
        </div>

        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">SOL Balance</span>
            <Activity size={16} className="text-purple-400" />
          </div>
          <div className="font-bold text-white text-lg">
            {formatSOL(portfolioData.sol_balance)}
          </div>
          <div className="text-xs text-gray-400 mt-1">
            Native SOL
          </div>
        </div>

        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">USD Value</span>
            <DollarSign size={16} className="text-green-400" />
          </div>
          <div className="font-bold text-white text-lg">
            {formatCurrency(portfolioData.total_usd_value)}
          </div>
          <div className="text-xs text-gray-400 mt-1">
            @ {formatCurrency(portfolioData.sol_price_usd)}/SOL
          </div>
        </div>

        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">Trading Mode</span>
            {portfolioData.trading_mode === 'PAPER' ? (
              <TrendingDown size={16} className="text-yellow-400" />
            ) : (
              <TrendingUp size={16} className="text-green-400" />
            )}
          </div>
          <div className={`font-bold ${portfolioData.trading_mode === 'PAPER' ? 'text-yellow-400' : 'text-green-400'}`}>
            {portfolioData.trading_mode}
          </div>
          <div className="text-xs text-gray-400 mt-1">
            {portfolioData.trading_mode === 'PAPER' ? 'Simulation' : 'Real Trading'}
          </div>
        </div>
      </div>

      {/* Wallet Address */}
      <div className="bg-gray-700 rounded-lg p-4 mb-6">
        <div className="text-gray-400 text-sm mb-2">Wallet Address</div>
        <div className="font-mono text-white text-sm break-all">
          {portfolioData.wallet_address}
        </div>
        <button
          onClick={() => copyToClipboard(portfolioData.wallet_address)}
          className="mt-2 px-3 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 flex items-center gap-1"
        >
          <Copy size={12} />
          Copy Address
        </button>
      </div>

      {/* Active Positions Summary */}
      <div className="bg-gray-700 rounded-lg p-4">
        <h3 className="text-white font-bold mb-3">Portfolio Summary</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <div className="text-gray-400 text-sm">Active Positions</div>
            <div className="text-white font-bold text-lg">{portfolioData.active_positions_count}</div>
          </div>
          <div>
            <div className="text-gray-400 text-sm">Token Holdings</div>
            <div className="text-white font-bold text-lg">{portfolioData.token_balances?.length || 0}</div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PortfolioOverview;

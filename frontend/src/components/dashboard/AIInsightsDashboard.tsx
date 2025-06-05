import React from 'react';
import { TrendingUp, TrendingDown, ArrowRight } from 'lucide-react';
import InsightChart from '../charts/InsightChart';

const AIInsightsDashboard: React.FC = () => {
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center space-x-4">
          <button className="px-4 py-2 bg-slate-800/50 rounded-full text-sm hover:bg-slate-700/50 transition-colors">
            Staking
          </button>
          <button className="px-4 py-2 bg-slate-800/50 rounded-full text-sm hover:bg-slate-700/50 transition-colors">
            Trading
          </button>
        </div>
        <button className="px-4 py-2 bg-purple-600/20 text-purple-400 rounded-full text-sm hover:bg-purple-600/30 transition-colors">
          Connect Wallet
        </button>
      </div>

      <div className="mb-8">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-medium">Top Trading Assets</h2>
          <div className="flex items-center space-x-3">
            <select className="bg-slate-800/50 border-none rounded-lg px-3 py-1.5 text-sm">
              <option>24H</option>
              <option>7D</option>
              <option>30D</option>
            </select>
            <select className="bg-slate-800/50 border-none rounded-lg px-3 py-1.5 text-sm">
              <option>Proof of Stake</option>
              <option>Proof of Work</option>
            </select>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Asset Cards */}
          {[
            { name: 'Ethereum (ETH)', rate: '13.62%', change: '+2.25%', positive: true },
            { name: 'BNB Chain', rate: '12.72%', change: '+1.87%', positive: true },
            { name: 'Polygon (MATIC)', rate: '6.29%', change: '-1.03%', positive: false }
          ].map((asset, index) => (
            <div key={index} className="bg-slate-800/30 rounded-xl p-4 border border-slate-700/30">
              <div className="flex justify-between items-start mb-4">
                <div className="flex items-center">
                  <div className="w-8 h-8 rounded-lg bg-gradient-to-r from-blue-500 to-purple-500 mr-3"></div>
                  <span>{asset.name}</span>
                </div>
                <ArrowRight size={18} className="text-slate-400" />
              </div>
              <div className="space-y-2">
                <div className="text-2xl font-semibold">{asset.rate}</div>
                <div className={`flex items-center text-sm ${asset.positive ? 'text-green-400' : 'text-red-400'}`}>
                  {asset.positive ? <TrendingUp size={16} className="mr-1" /> : <TrendingDown size={16} className="mr-1" />}
                  {asset.change}
                </div>
              </div>
              <div className="mt-4 h-16">
                <InsightChart type="accuracy" />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Active Trading Section */}
      <div className="bg-slate-800/30 rounded-xl p-6 border border-slate-700/30">
        <div className="flex items-center justify-between mb-6">
          <div>
            <div className="text-sm text-slate-400 mb-1">Last Update - 45 minutes ago</div>
            <div className="flex items-center space-x-3">
              <h3 className="text-xl font-medium">Trade SOL/USDC</h3>
              <button className="text-sm text-blue-400 hover:text-blue-300">View Profile</button>
            </div>
          </div>
          <div className="flex space-x-2">
            <button className="px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg text-sm transition-colors">
              Long
            </button>
            <button className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm transition-colors">
              Short
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div>
            <div className="text-sm text-slate-400 mb-1">Current Price</div>
            <div className="text-2xl font-semibold">$41.99</div>
            <div className="text-sm text-red-400">-1.09%</div>
          </div>
          <div>
            <div className="text-sm text-slate-400 mb-1">24h Volume</div>
            <div className="text-2xl font-semibold">$1.24B</div>
            <div className="text-sm text-green-400">+12.5%</div>
          </div>
          <div>
            <div className="text-sm text-slate-400 mb-1">Success Rate</div>
            <div className="text-2xl font-semibold">76.5%</div>
            <div className="text-sm text-green-400">+5.2%</div>
          </div>
          <div>
            <div className="text-sm text-slate-400 mb-1">Profit/Loss</div>
            <div className="text-2xl font-semibold">+$3,245.78</div>
            <div className="text-sm text-green-400">Last 24h</div>
          </div>
        </div>

        <div className="mt-6">
          <div className="h-64">
            <InsightChart type="accuracy" />
          </div>
        </div>
      </div>
    </div>
  );
};

export default AIInsightsDashboard;
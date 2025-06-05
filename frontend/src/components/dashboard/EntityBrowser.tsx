import React, { useState } from 'react';
import { Search, Filter, ChevronDown, Sliders } from 'lucide-react';

const EntityBrowser: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'tokens' | 'wallets' | 'contracts' | 'exchanges'>('tokens');
  const [showFilters, setShowFilters] = useState(false);
  
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold">Entity Browser</h1>
        <div className="flex space-x-2">
          <div className="relative">
            <Search className="absolute left-3 top-2.5 text-slate-400" size={16} />
            <input
              type="text"
              placeholder="Search entities..."
              className="pl-9 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-sm w-64"
            />
          </div>
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`p-2 rounded-lg ${showFilters ? 'bg-blue-600' : 'bg-slate-700'} hover:bg-blue-700 transition-colors`}
          >
            <Filter size={20} />
          </button>
        </div>
      </div>
      
      <div className="border-b border-slate-700/50">
        <nav className="flex space-x-4" aria-label="Tabs">
          <button
            onClick={() => setActiveTab('tokens')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'tokens'
                ? 'border-blue-500 text-blue-500'
                : 'border-transparent hover:border-slate-700'
            }`}
          >
            Tokens
          </button>
          <button
            onClick={() => setActiveTab('wallets')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'wallets'
                ? 'border-blue-500 text-blue-500'
                : 'border-transparent hover:border-slate-700'
            }`}
          >
            Wallets
          </button>
          <button
            onClick={() => setActiveTab('contracts')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'contracts'
                ? 'border-blue-500 text-blue-500'
                : 'border-transparent hover:border-slate-700'
            }`}
          >
            Smart Contracts
          </button>
          <button
            onClick={() => setActiveTab('exchanges')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'exchanges'
                ? 'border-blue-500 text-blue-500'
                : 'border-transparent hover:border-slate-700'
            }`}
          >
            Exchanges
          </button>
        </nav>
      </div>
      
      {showFilters && (
        <div className="bg-slate-800/50 rounded-lg p-4 border border-slate-700/50">
          <div className="flex justify-between items-center mb-4">
            <h3 className="font-medium">Filters</h3>
            <button className="text-sm text-blue-500 hover:text-blue-400">
              Reset
            </button>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm text-slate-400 mb-1">Risk Score</label>
              <div className="flex items-center space-x-2">
                <input
                  type="range"
                  min="0"
                  max="10"
                  defaultValue="5"
                  className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer"
                />
                <span className="text-sm">5+</span>
              </div>
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Activity Period</label>
              <select className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm">
                <option>Last 24 Hours</option>
                <option>Last 7 Days</option>
                <option>Last 30 Days</option>
                <option>All Time</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Sort By</label>
              <select className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm">
                <option>Activity (High to Low)</option>
                <option>Risk Score (High to Low)</option>
                <option>Creation Date (Newest)</option>
                <option>Name (A-Z)</option>
              </select>
            </div>
          </div>
        </div>
      )}
      
      <div className="bg-slate-800/50 rounded-xl overflow-hidden border border-slate-700/50">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-slate-700/50">
            <thead className="bg-slate-800">
              <tr>
                <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                  Name
                </th>
                <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                  ID
                </th>
                <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                  Type
                </th>
                <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                  <div className="flex items-center">
                    Risk Score
                    <ChevronDown size={14} className="ml-1" />
                  </div>
                </th>
                <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                  <div className="flex items-center">
                    Activity
                    <ChevronDown size={14} className="ml-1" />
                  </div>
                </th>
                <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                  Last Seen
                </th>
                <th scope="col" className="relative px-6 py-3">
                  <span className="sr-only">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/50">
              {getEntityData(activeTab).map((entity, idx) => (
                <tr key={idx} className="hover:bg-slate-700/30 cursor-pointer">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <div className={`flex-shrink-0 w-8 h-8 rounded-lg ${getEntityColorClass(entity.type)} flex items-center justify-center`}>
                        {entity.icon}
                      </div>
                      <div className="ml-3">
                        <div className="text-sm font-medium">{entity.name}</div>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-slate-400 truncate max-w-[150px]">{entity.id}</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm">{entity.type}</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className={`text-sm ${getRiskClass(entity.riskScore)}`}>{entity.riskScore}/10</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="w-32 bg-slate-700/50 rounded-full h-2">
                      <div
                        className="bg-blue-500 h-2 rounded-full"
                        style={{ width: `${entity.activity}%` }}
                      ></div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-400">
                    {entity.lastSeen}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm">
                    <button className="text-blue-500 hover:text-blue-400">Details</button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="bg-slate-800 px-4 py-3 flex items-center justify-between border-t border-slate-700/50">
          <div className="flex-1 flex justify-between items-center">
            <div>
              <p className="text-sm text-slate-400">
                Showing <span className="font-medium">1</span> to <span className="font-medium">10</span> of{' '}
                <span className="font-medium">97</span> results
              </p>
            </div>
            <div className="flex space-x-2">
              <button className="relative inline-flex items-center px-4 py-2 border border-slate-700 text-sm font-medium rounded-md bg-slate-800 hover:bg-slate-700">
                Previous
              </button>
              <button className="relative inline-flex items-center px-4 py-2 border border-slate-700 text-sm font-medium rounded-md bg-slate-800 hover:bg-slate-700">
                Next
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper functions and mock data
const getEntityColorClass = (type: string): string => {
  switch (type.toLowerCase()) {
    case 'token':
      return 'bg-purple-500';
    case 'wallet':
      return 'bg-blue-500';
    case 'contract':
      return 'bg-green-500';
    case 'exchange':
      return 'bg-yellow-500';
    default:
      return 'bg-slate-500';
  }
};

const getRiskClass = (score: number): string => {
  if (score >= 7) return 'text-red-500';
  if (score >= 4) return 'text-yellow-500';
  return 'text-green-500';
};

const getEntityData = (type: string) => {
  switch (type) {
    case 'tokens':
      return [
        { name: 'Solana', id: 'SOL', type: 'Token', riskScore: 1, activity: 95, lastSeen: 'Now', icon: 'SOL' },
        { name: 'USDC', id: 'USDC', type: 'Token', riskScore: 1, activity: 90, lastSeen: 'Now', icon: 'USD' },
        { name: 'Raydium', id: 'RAY', type: 'Token', riskScore: 3, activity: 75, lastSeen: '5m ago', icon: 'RAY' },
        { name: 'Serum', id: 'SRM', type: 'Token', riskScore: 2, activity: 65, lastSeen: '15m ago', icon: 'SRM' },
        { name: 'Star Atlas', id: 'ATLAS', type: 'Token', riskScore: 4, activity: 45, lastSeen: '2h ago', icon: 'ATL' },
        { name: 'Mango', id: 'MNGO', type: 'Token', riskScore: 3, activity: 40, lastSeen: '4h ago', icon: 'MNG' },
        { name: 'Bonfida', id: 'FIDA', type: 'Token', riskScore: 3, activity: 35, lastSeen: '6h ago', icon: 'FID' },
        { name: 'DeFi Land', id: 'DFL', type: 'Token', riskScore: 5, activity: 30, lastSeen: '12h ago', icon: 'DFL' },
        { name: 'Orca', id: 'ORCA', type: 'Token', riskScore: 2, activity: 70, lastSeen: '30m ago', icon: 'ORC' },
        { name: 'Mercurial', id: 'MER', type: 'Token', riskScore: 4, activity: 25, lastSeen: '1d ago', icon: 'MER' },
      ];
    case 'wallets':
      return [
        { name: 'Whale Wallet Alpha', id: '8xf3a...j7f2', type: 'Wallet', riskScore: 2, activity: 85, lastSeen: '6m ago', icon: 'WA' },
        { name: 'Institutional Fund', id: '3jk2a...p9d3', type: 'Wallet', riskScore: 1, activity: 75, lastSeen: '1h ago', icon: 'IF' },
        { name: 'Validator Node', id: '7kl2b...f4s5', type: 'Wallet', riskScore: 1, activity: 95, lastSeen: 'Now', icon: 'VN' },
        { name: 'Exchange Hot Wallet', id: '2xb7c...k3j4', type: 'Wallet', riskScore: 3, activity: 90, lastSeen: '10m ago', icon: 'EH' },
        { name: 'Suspicious Actor', id: '9mn5s...h2j3', type: 'Wallet', riskScore: 8, activity: 65, lastSeen: '3h ago', icon: 'SA' },
        { name: 'DEX LP Provider', id: '4gd2s...l9f2', type: 'Wallet', riskScore: 2, activity: 60, lastSeen: '5h ago', icon: 'LP' },
        { name: 'NFT Collector', id: '6jk3s...m4d2', type: 'Wallet', riskScore: 3, activity: 40, lastSeen: '2d ago', icon: 'NC' },
        { name: 'Trader Beta', id: '1fd4s...j7k2', type: 'Wallet', riskScore: 4, activity: 70, lastSeen: '8h ago', icon: 'TB' },
        { name: 'Staking Pool', id: '5hg3d...n5j2', type: 'Wallet', riskScore: 1, activity: 80, lastSeen: '4h ago', icon: 'SP' },
        { name: 'Game Treasury', id: '3kf7d...p2j4', type: 'Wallet', riskScore: 2, activity: 35, lastSeen: '1d ago', icon: 'GT' },
      ];
    case 'contracts':
      return [
        { name: 'Lending Protocol', id: 'prog...4jf2', type: 'Contract', riskScore: 3, activity: 85, lastSeen: '15m ago', icon: 'LP' },
        { name: 'DEX Router', id: 'prog...8kd3', type: 'Contract', riskScore: 2, activity: 95, lastSeen: 'Now', icon: 'DR' },
        { name: 'Stake Pool', id: 'prog...2jf5', type: 'Contract', riskScore: 1, activity: 80, lastSeen: '30m ago', icon: 'SP' },
        { name: 'NFT Marketplace', id: 'prog...5kf7', type: 'Contract', riskScore: 3, activity: 60, lastSeen: '2h ago', icon: 'NM' },
        { name: 'Unknown Protocol', id: 'prog...9jf2', type: 'Contract', riskScore: 7, activity: 45, lastSeen: '1d ago', icon: 'UP' },
        { name: 'Yield Aggregator', id: 'prog...3kd8', type: 'Contract', riskScore: 4, activity: 70, lastSeen: '5h ago', icon: 'YA' },
        { name: 'Governance', id: 'prog...1jd9', type: 'Contract', riskScore: 1, activity: 50, lastSeen: '1d ago', icon: 'GV' },
        { name: 'Bridge Contract', id: 'prog...7kf2', type: 'Contract', riskScore: 5, activity: 65, lastSeen: '8h ago', icon: 'BC' },
        { name: 'Escrow Service', id: 'prog...4jk2', type: 'Contract', riskScore: 3, activity: 40, lastSeen: '3d ago', icon: 'ES' },
        { name: 'IDO Platform', id: 'prog...2kd7', type: 'Contract', riskScore: 4, activity: 30, lastSeen: '5d ago', icon: 'IP' },
      ];
    case 'exchanges':
      return [
        { name: 'Centralized Exchange', id: 'exch...3jf5', type: 'Exchange', riskScore: 2, activity: 90, lastSeen: 'Now', icon: 'CE' },
        { name: 'Decentralized Exchange', id: 'exch...7kd2', type: 'Exchange', riskScore: 3, activity: 85, lastSeen: '5m ago', icon: 'DE' },
        { name: 'Liquidity Pool', id: 'exch...1jf9', type: 'Exchange', riskScore: 2, activity: 80, lastSeen: '15m ago', icon: 'LP' },
        { name: 'OTC Desk', id: 'exch...5kd3', type: 'Exchange', riskScore: 4, activity: 50, lastSeen: '2d ago', icon: 'OT' },
        { name: 'Token Swap', id: 'exch...9jf1', type: 'Exchange', riskScore: 3, activity: 60, lastSeen: '1d ago', icon: 'TS' },
        { name: 'Cross-Chain Bridge', id: 'exch...2kd8', type: 'Exchange', riskScore: 5, activity: 70, lastSeen: '8h ago', icon: 'CB' },
        { name: 'Fiat On-Ramp', id: 'exch...4jf3', type: 'Exchange', riskScore: 2, activity: 40, lastSeen: '3d ago', icon: 'FR' },
        { name: 'Aggregator', id: 'exch...6kd1', type: 'Exchange', riskScore: 3, activity: 75, lastSeen: '4h ago', icon: 'AG' },
        { name: 'Yield Platform', id: 'exch...8jf5', type: 'Exchange', riskScore: 3, activity: 55, lastSeen: '1d ago', icon: 'YP' },
        { name: 'NFT Exchange', id: 'exch...2jf7', type: 'Exchange', riskScore: 4, activity: 35, lastSeen: '5d ago', icon: 'NE' },
      ];
    default:
      return [];
  }
};

export default EntityBrowser;
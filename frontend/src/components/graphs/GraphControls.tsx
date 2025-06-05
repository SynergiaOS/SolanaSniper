import React, { useState } from 'react';
import { Eye, EyeOff } from 'lucide-react';

const GraphControls: React.FC = () => {
  const [filters, setFilters] = useState({
    wallets: true,
    tokens: true,
    contracts: true,
    exchanges: true
  });

  const toggleFilter = (key: keyof typeof filters) => {
    setFilters(prev => ({
      ...prev,
      [key]: !prev[key]
    }));
  };

  return (
    <div className="space-y-4">
      <h3 className="font-medium text-sm uppercase tracking-wider text-slate-400">Filters</h3>
      
      <div className="space-y-2">
        <div className="font-medium text-sm mb-2">Node Types</div>
        
        <div className="flex items-center justify-between p-2 hover:bg-slate-700/30 rounded-lg cursor-pointer"
             onClick={() => toggleFilter('wallets')}>
          <div className="flex items-center">
            <div className="w-3 h-3 rounded-full bg-blue-500 mr-2"></div>
            <span>Wallets</span>
          </div>
          {filters.wallets ? <Eye size={16} /> : <EyeOff size={16} className="text-slate-500" />}
        </div>
        
        <div className="flex items-center justify-between p-2 hover:bg-slate-700/30 rounded-lg cursor-pointer"
             onClick={() => toggleFilter('tokens')}>
          <div className="flex items-center">
            <div className="w-3 h-3 rounded-full bg-purple-500 mr-2"></div>
            <span>Tokens</span>
          </div>
          {filters.tokens ? <Eye size={16} /> : <EyeOff size={16} className="text-slate-500" />}
        </div>
        
        <div className="flex items-center justify-between p-2 hover:bg-slate-700/30 rounded-lg cursor-pointer"
             onClick={() => toggleFilter('contracts')}>
          <div className="flex items-center">
            <div className="w-3 h-3 rounded-full bg-green-500 mr-2"></div>
            <span>Contracts</span>
          </div>
          {filters.contracts ? <Eye size={16} /> : <EyeOff size={16} className="text-slate-500" />}
        </div>
        
        <div className="flex items-center justify-between p-2 hover:bg-slate-700/30 rounded-lg cursor-pointer"
             onClick={() => toggleFilter('exchanges')}>
          <div className="flex items-center">
            <div className="w-3 h-3 rounded-full bg-yellow-500 mr-2"></div>
            <span>Exchanges</span>
          </div>
          {filters.exchanges ? <Eye size={16} /> : <EyeOff size={16} className="text-slate-500" />}
        </div>
      </div>
      
      <div className="border-t border-slate-700/50 pt-4">
        <h3 className="font-medium text-sm mb-2">Connection Strength</h3>
        <input 
          type="range" 
          min="0" 
          max="100" 
          defaultValue="30"
          className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer"
        />
        <div className="flex justify-between text-xs text-slate-400 mt-1">
          <span>Weak</span>
          <span>Strong</span>
        </div>
      </div>
      
      <div className="border-t border-slate-700/50 pt-4">
        <h3 className="font-medium text-sm mb-2">Time Range</h3>
        <div className="flex items-center space-x-2">
          <select className="bg-slate-700 border border-slate-600 rounded-lg px-2 py-1 text-sm flex-1">
            <option>Last 24 Hours</option>
            <option>Last 7 Days</option>
            <option>Last 30 Days</option>
            <option>Custom Range</option>
          </select>
        </div>
      </div>
      
      <div className="border-t border-slate-700/50 pt-4">
        <h3 className="font-medium text-sm mb-2">Layout</h3>
        <div className="grid grid-cols-2 gap-2">
          <button className="bg-blue-600 text-white py-1 px-2 rounded text-sm">Force-Directed</button>
          <button className="bg-slate-700 hover:bg-slate-600 py-1 px-2 rounded text-sm">Circular</button>
          <button className="bg-slate-700 hover:bg-slate-600 py-1 px-2 rounded text-sm">Hierarchical</button>
          <button className="bg-slate-700 hover:bg-slate-600 py-1 px-2 rounded text-sm">Radial</button>
        </div>
      </div>
      
      <div className="pt-4">
        <button className="w-full bg-slate-700 hover:bg-slate-600 py-2 rounded-lg text-sm">
          Reset Filters
        </button>
      </div>
    </div>
  );
};

export default GraphControls;
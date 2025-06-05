import React, { useState } from 'react';
import { Bell, BellOff, Settings, Plus, Trash2, AlertTriangle, CheckCircle, Clock, Info } from 'lucide-react';

const SmartAlerts: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'active' | 'triggered' | 'configure'>('active');
  const [showNewAlert, setShowNewAlert] = useState(false);
  
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold">Smart Alerts</h1>
        <div className="flex space-x-2">
          <button
            onClick={() => setShowNewAlert(!showNewAlert)}
            className="flex items-center space-x-1 px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm transition-colors"
          >
            <Plus size={16} />
            <span>New Alert</span>
          </button>
          <button className="p-2 bg-slate-700 hover:bg-slate-600 rounded-lg transition-colors">
            <Settings size={20} />
          </button>
        </div>
      </div>
      
      <div className="border-b border-slate-700/50">
        <nav className="flex space-x-4" aria-label="Tabs">
          <button
            onClick={() => setActiveTab('active')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'active'
                ? 'border-blue-500 text-blue-500'
                : 'border-transparent hover:border-slate-700'
            }`}
          >
            Active Alerts
          </button>
          <button
            onClick={() => setActiveTab('triggered')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'triggered'
                ? 'border-blue-500 text-blue-500'
                : 'border-transparent hover:border-slate-700'
            }`}
          >
            Triggered History
          </button>
          <button
            onClick={() => setActiveTab('configure')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'configure'
                ? 'border-blue-500 text-blue-500'
                : 'border-transparent hover:border-slate-700'
            }`}
          >
            Configure
          </button>
        </nav>
      </div>
      
      {showNewAlert && (
        <div className="bg-slate-800/80 rounded-xl p-4 border border-blue-500/30">
          <div className="flex justify-between items-center mb-4">
            <h3 className="font-medium">Create New Alert</h3>
            <button onClick={() => setShowNewAlert(false)} className="text-slate-400 hover:text-slate-300">
              <Trash2 size={16} />
            </button>
          </div>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-slate-400 mb-1">Alert Type</label>
              <select className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm">
                <option>Whale Movement</option>
                <option>Price Change</option>
                <option>Trading Pattern</option>
                <option>Smart Contract Activity</option>
                <option>Custom AI Detection</option>
              </select>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-slate-400 mb-1">Entity Type</label>
                <select className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm">
                  <option>Token</option>
                  <option>Wallet</option>
                  <option>Contract</option>
                  <option>Exchange</option>
                </select>
              </div>
              
              <div>
                <label className="block text-sm text-slate-400 mb-1">Entity</label>
                <div className="relative">
                  <input
                    type="text"
                    placeholder="Search or enter address..."
                    className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm"
                  />
                </div>
              </div>
            </div>
            
            <div>
              <label className="block text-sm text-slate-400 mb-1">Threshold</label>
              <div className="flex space-x-2">
                <select className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm w-1/3">
                  <option>Greater than</option>
                  <option>Less than</option>
                  <option>Equal to</option>
                  <option>Changes by</option>
                </select>
                <input
                  type="text"
                  placeholder="Value"
                  className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm w-1/3"
                />
                <select className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm w-1/3">
                  <option>SOL</option>
                  <option>USD</option>
                  <option>%</option>
                  <option>Units</option>
                </select>
              </div>
            </div>
            
            <div>
              <label className="block text-sm text-slate-400 mb-1">Notification Method</label>
              <div className="flex space-x-2">
                <label className="flex items-center space-x-2">
                  <input type="checkbox" className="rounded bg-slate-700 border-slate-600 text-blue-500" defaultChecked />
                  <span className="text-sm">In-App</span>
                </label>
                <label className="flex items-center space-x-2">
                  <input type="checkbox" className="rounded bg-slate-700 border-slate-600 text-blue-500" defaultChecked />
                  <span className="text-sm">Email</span>
                </label>
                <label className="flex items-center space-x-2">
                  <input type="checkbox" className="rounded bg-slate-700 border-slate-600 text-blue-500" />
                  <span className="text-sm">Webhook</span>
                </label>
              </div>
            </div>
            
            <div className="pt-2 flex justify-end space-x-2">
              <button
                onClick={() => setShowNewAlert(false)}
                className="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm transition-colors"
              >
                Cancel
              </button>
              <button className="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm transition-colors">
                Create Alert
              </button>
            </div>
          </div>
        </div>
      )}
      
      {activeTab === 'active' && (
        <div className="space-y-4">
          {getActiveAlerts().map((alert, idx) => (
            <div key={idx} className="bg-slate-800/50 rounded-xl p-4 border border-slate-700/50 hover:border-slate-600/80 transition-colors">
              <div className="flex items-start">
                <div className={`flex-shrink-0 p-2 rounded-lg ${getAlertTypeClass(alert.type)} mr-3`}>
                  {getAlertTypeIcon(alert.type)}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex justify-between items-start">
                    <div>
                      <h3 className="font-medium">{alert.name}</h3>
                      <p className="text-sm text-slate-400">{alert.description}</p>
                    </div>
                    <div className="flex space-x-2">
                      <button className="p-1 hover:bg-slate-700/50 rounded-lg transition-colors">
                        <Settings size={16} />
                      </button>
                      <button className="p-1 hover:bg-slate-700/50 rounded-lg transition-colors">
                        <BellOff size={16} />
                      </button>
                    </div>
                  </div>
                  
                  <div className="mt-3 flex flex-wrap items-center gap-2">
                    <div className="bg-slate-700/30 px-2 py-1 rounded text-xs">
                      {alert.entity.type}: {alert.entity.name}
                    </div>
                    <div className="bg-slate-700/30 px-2 py-1 rounded text-xs">
                      Threshold: {alert.threshold}
                    </div>
                    <div className="bg-slate-700/30 px-2 py-1 rounded text-xs">
                      Created: {alert.created}
                    </div>
                    <div className="ml-auto flex items-center text-xs text-slate-400">
                      <Clock size={14} className="mr-1" />
                      <span>Last checked: {alert.lastChecked}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
      
      {activeTab === 'triggered' && (
        <div className="bg-slate-800/50 rounded-xl overflow-hidden border border-slate-700/50">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-slate-700/50">
              <thead className="bg-slate-800">
                <tr>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                    Alert
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                    Entity
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                    Triggered Value
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                    Timestamp
                  </th>
                  <th scope="col" className="relative px-6 py-3">
                    <span className="sr-only">Actions</span>
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700/50">
                {getTriggeredAlerts().map((alert, idx) => (
                  <tr key={idx} className="hover:bg-slate-700/30">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <div className={`flex-shrink-0 w-8 h-8 rounded-full ${getAlertTypeClass(alert.type)} flex items-center justify-center`}>
                          {getAlertTypeIcon(alert.type)}
                        </div>
                        <div className="ml-3">
                          <div className="text-sm font-medium">{alert.name}</div>
                          <div className="text-xs text-slate-400">{alert.type}</div>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm">{alert.entity.name}</div>
                      <div className="text-xs text-slate-400">{alert.entity.type}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm">{alert.triggeredValue}</div>
                      <div className="text-xs text-slate-400">Threshold: {alert.threshold}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      {alert.timestamp}
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
                  Showing <span className="font-medium">1</span> to <span className="font-medium">5</span> of{' '}
                  <span className="font-medium">12</span> results
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
      )}
      
      {activeTab === 'configure' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700/50">
            <h3 className="font-medium mb-4">Notification Settings</h3>
            <div className="space-y-4">
              <div>
                <label className="flex items-center justify-between">
                  <span className="text-sm">Email Notifications</span>
                  <div className="relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full">
                    <input type="checkbox" className="absolute w-6 h-6 transition duration-200 ease-in-out transform bg-white border rounded-full appearance-none cursor-pointer peer checked:translate-x-6 checked:bg-blue-500 peer-checked:border-blue-500" defaultChecked />
                    <span className="absolute w-12 h-6 transition duration-200 ease-in-out rounded-full cursor-pointer peer-checked:bg-blue-600/20 bg-slate-700"></span>
                  </div>
                </label>
                <input
                  type="email"
                  placeholder="your@email.com"
                  className="mt-2 w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm"
                />
              </div>
              
              <div>
                <label className="flex items-center justify-between">
                  <span className="text-sm">Webhook Notifications</span>
                  <div className="relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full">
                    <input type="checkbox" className="absolute w-6 h-6 transition duration-200 ease-in-out transform bg-white border rounded-full appearance-none cursor-pointer peer checked:translate-x-6 checked:bg-blue-500 peer-checked:border-blue-500" />
                    <span className="absolute w-12 h-6 transition duration-200 ease-in-out rounded-full cursor-pointer peer-checked:bg-blue-600/20 bg-slate-700"></span>
                  </div>
                </label>
                <input
                  type="url"
                  placeholder="https://your-webhook-url.com"
                  className="mt-2 w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm"
                />
              </div>
              
              <div>
                <label className="flex items-center justify-between">
                  <span className="text-sm">Push Notifications</span>
                  <div className="relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full">
                    <input type="checkbox" className="absolute w-6 h-6 transition duration-200 ease-in-out transform bg-white border rounded-full appearance-none cursor-pointer peer checked:translate-x-6 checked:bg-blue-500 peer-checked:border-blue-500" defaultChecked />
                    <span className="absolute w-12 h-6 transition duration-200 ease-in-out rounded-full cursor-pointer peer-checked:bg-blue-600/20 bg-slate-700"></span>
                  </div>
                </label>
                <p className="mt-1 text-xs text-slate-400">Receive push notifications in your browser</p>
              </div>
              
              <div>
                <label className="flex items-center justify-between">
                  <span className="text-sm">Sound Alerts</span>
                  <div className="relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full">
                    <input type="checkbox" className="absolute w-6 h-6 transition duration-200 ease-in-out transform bg-white border rounded-full appearance-none cursor-pointer peer checked:translate-x-6 checked:bg-blue-500 peer-checked:border-blue-500" />
                    <span className="absolute w-12 h-6 transition duration-200 ease-in-out rounded-full cursor-pointer peer-checked:bg-blue-600/20 bg-slate-700"></span>
                  </div>
                </label>
                <p className="mt-1 text-xs text-slate-400">Play sound when alerts are triggered</p>
              </div>
            </div>
          </div>
          
          <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700/50">
            <h3 className="font-medium mb-4">Alert Rules</h3>
            <div className="space-y-4">
              <div>
                <label className="flex items-center justify-between">
                  <span className="text-sm">AI-Generated Alerts</span>
                  <div className="relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full">
                    <input type="checkbox" className="absolute w-6 h-6 transition duration-200 ease-in-out transform bg-white border rounded-full appearance-none cursor-pointer peer checked:translate-x-6 checked:bg-blue-500 peer-checked:border-blue-500" defaultChecked />
                    <span className="absolute w-12 h-6 transition duration-200 ease-in-out rounded-full cursor-pointer peer-checked:bg-blue-600/20 bg-slate-700"></span>
                  </div>
                </label>
                <p className="mt-1 text-xs text-slate-400">Receive alerts generated by AI based on your activity</p>
              </div>
              
              <div>
                <label className="block text-sm mb-1">AI Alert Sensitivity</label>
                <div className="flex items-center space-x-2">
                  <input
                    type="range"
                    min="0"
                    max="100"
                    defaultValue="70"
                    className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer"
                  />
                  <span className="text-sm">70%</span>
                </div>
                <p className="mt-1 text-xs text-slate-400">Higher values result in more alerts</p>
              </div>
              
              <div>
                <label className="block text-sm mb-1">Alert Frequency</label>
                <select className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm">
                  <option>Immediate</option>
                  <option>Hourly Digest</option>
                  <option>Daily Digest</option>
                  <option>Weekly Digest</option>
                </select>
              </div>
              
              <div>
                <label className="flex items-center justify-between">
                  <span className="text-sm">Repeated Alert Throttling</span>
                  <div className="relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full">
                    <input type="checkbox" className="absolute w-6 h-6 transition duration-200 ease-in-out transform bg-white border rounded-full appearance-none cursor-pointer peer checked:translate-x-6 checked:bg-blue-500 peer-checked:border-blue-500" defaultChecked />
                    <span className="absolute w-12 h-6 transition duration-200 ease-in-out rounded-full cursor-pointer peer-checked:bg-blue-600/20 bg-slate-700"></span>
                  </div>
                </label>
                <p className="mt-1 text-xs text-slate-400">Prevent multiple similar alerts in short timeframes</p>
              </div>
            </div>
            
            <div className="mt-6">
              <button className="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 rounded-lg text-sm transition-colors">
                Save Settings
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Helper functions and mock data
const getAlertTypeClass = (type: string): string => {
  switch (type.toLowerCase()) {
    case 'whale movement':
      return 'bg-blue-500/20 text-blue-300';
    case 'price change':
      return 'bg-green-500/20 text-green-300';
    case 'trading pattern':
      return 'bg-purple-500/20 text-purple-300';
    case 'smart contract activity':
      return 'bg-yellow-500/20 text-yellow-300';
    case 'custom ai detection':
      return 'bg-indigo-500/20 text-indigo-300';
    default:
      return 'bg-slate-500/20 text-slate-300';
  }
};

const getAlertTypeIcon = (type: string) => {
  switch (type.toLowerCase()) {
    case 'whale movement':
      return <Bell size={20} />;
    case 'price change':
      return <TrendingUp size={20} />;
    case 'trading pattern':
      return <Activity size={20} />;
    case 'smart contract activity':
      return <AlertTriangle size={20} />;
    case 'custom ai detection':
      return <Brain size={20} />;
    default:
      return <Info size={20} />;
  }
};

const getActiveAlerts = () => [
  {
    name: 'Large SOL Movement',
    type: 'Whale Movement',
    description: 'Alert when more than 10,000 SOL moves from monitored wallets',
    entity: { name: 'SOL', type: 'Token' },
    threshold: '> 10,000 SOL',
    created: '2 days ago',
    lastChecked: '5 minutes ago',
  },
  {
    name: 'Price Volatility Alert',
    type: 'Price Change',
    description: 'Alert when price changes by more than 8% in any 1-hour period',
    entity: { name: 'Raydium', type: 'Token' },
    threshold: '> 8% in 1h',
    created: '5 days ago',
    lastChecked: '2 minutes ago',
  },
  {
    name: 'Contract Interaction Spike',
    type: 'Smart Contract Activity',
    description: 'Alert when unusual transaction volume occurs on target contract',
    entity: { name: 'Lending Protocol', type: 'Contract' },
    threshold: '> 200% normal volume',
    created: '1 week ago',
    lastChecked: '10 minutes ago',
  },
  {
    name: 'Whale Wallet Accumulation',
    type: 'Trading Pattern',
    description: 'Alert when specific wallet begins accumulating target token',
    entity: { name: 'Whale Wallet Alpha', type: 'Wallet' },
    threshold: '> 5% increase in holdings',
    created: '3 days ago',
    lastChecked: '15 minutes ago',
  },
  {
    name: 'Unusual Token Migration',
    type: 'Custom AI Detection',
    description: 'AI-generated alert for suspicious token migration patterns',
    entity: { name: 'All Tokens', type: 'Multiple' },
    threshold: 'AI Confidence > 80%',
    created: 'Yesterday',
    lastChecked: 'Just now',
  },
];

const getTriggeredAlerts = () => [
  {
    name: 'Large SOL Movement',
    type: 'Whale Movement',
    entity: { name: 'SOL', type: 'Token' },
    threshold: '> 10,000 SOL',
    triggeredValue: '25,450 SOL moved',
    timestamp: '2 hours ago',
  },
  {
    name: 'Price Volatility Alert',
    type: 'Price Change',
    entity: { name: 'Raydium', type: 'Token' },
    threshold: '> 8% in 1h',
    triggeredValue: '+12.3% in 45 minutes',
    timestamp: 'Yesterday, 15:30',
  },
  {
    name: 'Contract Interaction Spike',
    type: 'Smart Contract Activity',
    entity: { name: 'Lending Protocol', type: 'Contract' },
    threshold: '> 200% normal volume',
    triggeredValue: '342% of normal volume',
    timestamp: '3 days ago',
  },
  {
    name: 'Whale Wallet Accumulation',
    type: 'Trading Pattern',
    entity: { name: 'Whale Wallet Alpha', type: 'Wallet' },
    threshold: '> 5% increase in holdings',
    triggeredValue: '8.7% increase in 24h',
    timestamp: 'Last week',
  },
  {
    name: 'Unusual Token Migration',
    type: 'Custom AI Detection',
    entity: { name: 'Star Atlas', type: 'Token' },
    threshold: 'AI Confidence > 80%',
    triggeredValue: '92% confidence pattern',
    timestamp: '2 days ago',
  },
];

const Brain = (props: any) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    {...props}
  >
    <path d="M9.5 2A2.5 2.5 0 0 1 12 4.5v15a2.5 2.5 0 0 1-4.96.44 2.5 2.5 0 0 1-2.96-3.08 3 3 0 0 1-.34-5.58 2.5 2.5 0 0 1 1.32-4.24 2.5 2.5 0 0 1 1.98-3A2.5 2.5 0 0 1 9.5 2Z" />
    <path d="M14.5 2A2.5 2.5 0 0 0 12 4.5v15a2.5 2.5 0 0 0 4.96.44 2.5 2.5 0 0 0 2.96-3.08 3 3 0 0 0 .34-5.58 2.5 2.5 0 0 0-1.32-4.24 2.5 2.5 0 0 0-1.98-3A2.5 2.5 0 0 0 14.5 2Z" />
  </svg>
);

const Activity = (props: any) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    {...props}
  >
    <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
  </svg>
);

const TrendingUp = (props: any) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    {...props}
  >
    <polyline points="23 6 13.5 15.5 8.5 10.5 1 18" />
    <polyline points="17 6 23 6 23 12" />
  </svg>
);

export default SmartAlerts;
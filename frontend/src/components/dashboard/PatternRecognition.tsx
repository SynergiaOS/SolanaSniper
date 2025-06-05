import React, { useState } from 'react';
import { TrendingUp, TrendingDown, Eye, EyeOff, ChevronDown, ChevronUp, Info } from 'lucide-react';

const PatternRecognition: React.FC = () => {
  const [expandedPattern, setExpandedPattern] = useState<string | null>(null);
  const [showHistorical, setShowHistorical] = useState(true);
  
  const togglePattern = (id: string) => {
    setExpandedPattern(expandedPattern === id ? null : id);
  };
  
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold">Pattern Recognition</h1>
        <div className="flex space-x-2">
          <button
            onClick={() => setShowHistorical(!showHistorical)}
            className="flex items-center space-x-1 text-sm bg-slate-800 hover:bg-slate-700 px-3 py-2 rounded-lg transition-colors"
          >
            {showHistorical ? <Eye size={16} /> : <EyeOff size={16} />}
            <span>Historical Patterns</span>
          </button>
          <select className="bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-sm">
            <option>Last 24 Hours</option>
            <option>Last 7 Days</option>
            <option>Last 30 Days</option>
          </select>
        </div>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div className="col-span-full">
          <div className="bg-gradient-to-r from-blue-600/20 to-purple-600/20 rounded-xl p-1">
            <div className="bg-slate-800/90 rounded-lg p-4 border border-blue-500/30">
              <div className="flex justify-between items-start">
                <div className="flex items-center">
                  <div className="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center mr-3">
                    <span className="text-white font-semibold">AI</span>
                  </div>
                  <div>
                    <h3 className="font-medium text-blue-400">Pattern Alert</h3>
                    <p className="text-sm">Potential market-moving whale activity detected</p>
                  </div>
                </div>
                <div className="bg-blue-500/20 text-blue-300 text-xs font-medium px-2 py-1 rounded">
                  Live
                </div>
              </div>
              <div className="mt-3 pl-12">
                <p className="text-sm text-slate-300">
                  Our AI has detected unusual accumulation patterns across 3 major wallets,
                  targeting the same token group. Historical data suggests this behavior precedes
                  significant price movements within 24-48 hours.
                </p>
                <div className="mt-2 flex space-x-2">
                  <button className="text-sm text-blue-400 hover:text-blue-300">
                    View Details
                  </button>
                  <button className="text-sm text-slate-400 hover:text-slate-300">
                    Set Alert
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        {getPatternData().map((pattern) => (
          <div 
            key={pattern.id}
            className={`bg-slate-800/50 rounded-xl border border-slate-700/50 overflow-hidden transition-all duration-300 ${
              expandedPattern === pattern.id ? 'col-span-full' : ''
            }`}
          >
            <div 
              className="p-4 cursor-pointer hover:bg-slate-700/20 transition-colors"
              onClick={() => togglePattern(pattern.id)}
            >
              <div className="flex justify-between items-start">
                <div className="flex items-center">
                  <div className={`w-10 h-10 rounded-full ${pattern.trend === 'up' ? 'bg-green-500' : 'bg-red-500'} flex items-center justify-center mr-3`}>
                    {pattern.trend === 'up' ? <TrendingUp size={20} className="text-white" /> : <TrendingDown size={20} className="text-white" />}
                  </div>
                  <div>
                    <h3 className="font-medium">{pattern.name}</h3>
                    <div className="flex items-center text-sm text-slate-400">
                      <span>{pattern.type}</span>
                      <span className="mx-1">•</span>
                      <span>{pattern.confidence}% confidence</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center">
                  <div className={`text-xs font-medium px-2 py-1 rounded ${
                    pattern.status === 'active' 
                      ? 'bg-green-500/20 text-green-300' 
                      : pattern.status === 'developing' 
                        ? 'bg-yellow-500/20 text-yellow-300'
                        : 'bg-slate-500/20 text-slate-300'
                  }`}>
                    {pattern.status}
                  </div>
                  <button className="ml-2">
                    {expandedPattern === pattern.id ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
                  </button>
                </div>
              </div>
            </div>
            
            {expandedPattern === pattern.id && (
              <div className="px-4 pb-4">
                <div className="pt-2 pb-4">
                  <div className="bg-slate-700/30 rounded-lg p-4">
                    <p className="text-sm mb-4">{pattern.description}</p>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                      <div>
                        <h4 className="text-xs text-slate-400 mb-1">Detected</h4>
                        <p className="text-sm">{pattern.detected}</p>
                      </div>
                      <div>
                        <h4 className="text-xs text-slate-400 mb-1">Accuracy (Historical)</h4>
                        <p className="text-sm">{pattern.accuracy}%</p>
                      </div>
                      <div>
                        <h4 className="text-xs text-slate-400 mb-1">Avg. Impact</h4>
                        <p className={`text-sm ${pattern.trend === 'up' ? 'text-green-400' : 'text-red-400'}`}>
                          {pattern.impact}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
                
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                  <div className="lg:col-span-2 bg-slate-700/30 rounded-lg p-4 h-64 flex items-center justify-center">
                    <div className="text-center text-slate-400">
                      <div className="flex justify-center mb-2">
                        <Info size={24} />
                      </div>
                      <p>Pattern visualization would appear here</p>
                      <p className="text-xs mt-1">Showing price action, volume, and indicator signals</p>
                    </div>
                  </div>
                  
                  <div className="space-y-4">
                    <div>
                      <h4 className="text-sm font-medium mb-2">Key Entities Involved</h4>
                      <div className="space-y-2">
                        {pattern.entities.map((entity, idx) => (
                          <div key={idx} className="flex items-center bg-slate-700/20 p-2 rounded-lg hover:bg-slate-700/30 cursor-pointer">
                            <div className={`w-6 h-6 rounded-md ${getEntityColorClass(entity.type)} flex items-center justify-center mr-2`}>
                              <span className="text-xs text-white">{entity.icon}</span>
                            </div>
                            <div className="flex-1 min-w-0">
                              <p className="text-sm truncate">{entity.name}</p>
                            </div>
                            <div className="text-xs text-slate-400">{entity.role}</div>
                          </div>
                        ))}
                      </div>
                    </div>
                    
                    <div>
                      <h4 className="text-sm font-medium mb-2">Prediction</h4>
                      <div className="bg-slate-700/20 p-3 rounded-lg">
                        <div className="flex justify-between items-center mb-1">
                          <span className="text-sm">Projected Outcome</span>
                          <span className={`text-sm ${pattern.trend === 'up' ? 'text-green-400' : 'text-red-400'}`}>
                            {pattern.prediction.outcome}
                          </span>
                        </div>
                        <div className="flex justify-between items-center mb-1">
                          <span className="text-sm">Probability</span>
                          <span className="text-sm">{pattern.prediction.probability}%</span>
                        </div>
                        <div className="flex justify-between items-center">
                          <span className="text-sm">Time Frame</span>
                          <span className="text-sm">{pattern.prediction.timeFrame}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                
                <div className="flex justify-end space-x-2 mt-4">
                  <button className="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm transition-colors">
                    Create Alert
                  </button>
                  <button className="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm transition-colors">
                    Analyze Further
                  </button>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
      
      {showHistorical && (
        <div>
          <div className="flex justify-between items-center mb-3">
            <h2 className="text-lg font-medium">Historical Patterns</h2>
            <button className="text-sm text-blue-500 hover:text-blue-400">
              View All
            </button>
          </div>
          
          <div className="bg-slate-800/50 rounded-xl overflow-hidden border border-slate-700/50">
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-slate-700/50">
                <thead className="bg-slate-800">
                  <tr>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                      Pattern
                    </th>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                      Date Detected
                    </th>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                      Result
                    </th>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                      Accuracy
                    </th>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                      Impact
                    </th>
                    <th scope="col" className="relative px-6 py-3">
                      <span className="sr-only">Actions</span>
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-700/50">
                  {getHistoricalPatterns().map((pattern, idx) => (
                    <tr key={idx} className="hover:bg-slate-700/30">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex items-center">
                          <div className={`flex-shrink-0 w-8 h-8 rounded-full ${
                            pattern.result === 'Confirmed' 
                              ? 'bg-green-500/20 text-green-400' 
                              : pattern.result === 'Failed' 
                                ? 'bg-red-500/20 text-red-400'
                                : 'bg-yellow-500/20 text-yellow-400'
                          } flex items-center justify-center`}>
                            {pattern.result === 'Confirmed' 
                              ? <TrendingUp size={16} /> 
                              : pattern.result === 'Failed'
                                ? <TrendingDown size={16} />
                                : <Info size={16} />}
                          </div>
                          <div className="ml-3">
                            <div className="text-sm font-medium">{pattern.name}</div>
                            <div className="text-xs text-slate-400">{pattern.type}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm">
                        {pattern.date}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span className={`px-2 py-1 text-xs rounded-full ${
                          pattern.result === 'Confirmed' 
                            ? 'bg-green-500/20 text-green-400' 
                            : pattern.result === 'Failed' 
                              ? 'bg-red-500/20 text-red-400'
                              : 'bg-yellow-500/20 text-yellow-400'
                        }`}>
                          {pattern.result}
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm">
                        {pattern.accuracy}%
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm">
                        <span className={pattern.impact.startsWith('+') ? 'text-green-400' : 'text-red-400'}>
                          {pattern.impact}
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right text-sm">
                        <button className="text-blue-500 hover:text-blue-400">View</button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}
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

const getPatternData = () => [
  {
    id: 'pattern1',
    name: 'Bull Flag Formation',
    type: 'Price Pattern',
    trend: 'up',
    confidence: 92,
    status: 'active',
    detected: '2 hours ago',
    accuracy: 87,
    impact: '+12-15% price increase',
    description: 'A bullish continuation pattern showing consolidation after a strong upward movement. The pattern suggests accumulation before continuation of the upward trend.',
    entities: [
      { name: 'SOL', type: 'token', icon: 'SOL', role: 'Target' },
      { name: 'Whale Wallet Alpha', type: 'wallet', icon: 'WA', role: 'Buyer' },
      { name: 'DEX Router', type: 'contract', icon: 'DR', role: 'Facilitator' },
    ],
    prediction: {
      outcome: '+14.5% increase',
      probability: 87,
      timeFrame: '24-48 hours',
    },
  },
  {
    id: 'pattern2',
    name: 'Wash Trading',
    type: 'Trading Behavior',
    trend: 'down',
    confidence: 78,
    status: 'developing',
    detected: '5 hours ago',
    accuracy: 81,
    impact: 'Artificial volume +320%',
    description: 'Suspicious trading pattern indicating potential market manipulation through artificial volume creation between related wallets.',
    entities: [
      { name: 'New DeFi Token', type: 'token', icon: 'NDT', role: 'Target' },
      { name: 'Suspicious Actor', type: 'wallet', icon: 'SA', role: 'Manipulator' },
      { name: 'Decentralized Exchange', type: 'exchange', icon: 'DE', role: 'Venue' },
    ],
    prediction: {
      outcome: 'Price instability',
      probability: 78,
      timeFrame: '12-24 hours',
    },
  },
  {
    id: 'pattern3',
    name: 'Accumulation Phase',
    type: 'Whale Strategy',
    trend: 'up',
    confidence: 85,
    status: 'active',
    detected: '1 day ago',
    accuracy: 79,
    impact: '+25-30% price increase',
    description: 'Large wallets gradually accumulating significant token amounts while maintaining low price impact, typically preceding major price movements.',
    entities: [
      { name: 'Raydium', type: 'token', icon: 'RAY', role: 'Target' },
      { name: 'Institutional Fund', type: 'wallet', icon: 'IF', role: 'Accumulator' },
      { name: 'OTC Desk', type: 'exchange', icon: 'OT', role: 'Facilitator' },
    ],
    prediction: {
      outcome: '+27% increase',
      probability: 85,
      timeFrame: '7-14 days',
    },
  },
  {
    id: 'pattern4',
    name: 'Supply Shock',
    type: 'Token Economics',
    trend: 'up',
    confidence: 82,
    status: 'developing',
    detected: '3 hours ago',
    accuracy: 73,
    impact: '+18-22% price increase',
    description: 'Significant reduction in circulating supply due to token lockups, staking, or burning, creating upward price pressure due to reduced selling pressure.',
    entities: [
      { name: 'Orca', type: 'token', icon: 'ORC', role: 'Target' },
      { name: 'Stake Pool', type: 'contract', icon: 'SP', role: 'Lock Mechanism' },
      { name: 'Governance', type: 'contract', icon: 'GV', role: 'Decision Maker' },
    ],
    prediction: {
      outcome: '+20% increase',
      probability: 82,
      timeFrame: '3-5 days',
    },
  },
  {
    id: 'pattern5',
    name: 'Distribution Phase',
    type: 'Whale Strategy',
    trend: 'down',
    confidence: 88,
    status: 'active',
    detected: '12 hours ago',
    accuracy: 84,
    impact: '-15-20% price decrease',
    description: 'Large holder gradually selling significant token amounts across multiple venues to minimize price impact, typically preceding market correction.',
    entities: [
      { name: 'Serum', type: 'token', icon: 'SRM', role: 'Target' },
      { name: 'Exchange Hot Wallet', type: 'wallet', icon: 'EH', role: 'Distributor' },
      { name: 'Centralized Exchange', type: 'exchange', icon: 'CE', role: 'Primary Venue' },
    ],
    prediction: {
      outcome: '-18% decrease',
      probability: 88,
      timeFrame: '2-4 days',
    },
  },
  {
    id: 'pattern6',
    name: 'Pre-announcement Activity',
    type: 'Insider Trading',
    trend: 'up',
    confidence: 75,
    status: 'historical',
    detected: '2 days ago',
    accuracy: 69,
    impact: '+35-40% price increase',
    description: 'Unusual trading activity preceding public announcements, suggesting potential information asymmetry or insider knowledge.',
    entities: [
      { name: 'Mango', type: 'token', icon: 'MNG', role: 'Target' },
      { name: 'Trader Beta', type: 'wallet', icon: 'TB', role: 'Early Buyer' },
      { name: 'IDO Platform', type: 'contract', icon: 'IP', role: 'Launch Platform' },
    ],
    prediction: {
      outcome: '+38% increase',
      probability: 75,
      timeFrame: 'After announcement',
    },
  },
];

const getHistoricalPatterns = () => [
  { name: 'Bull Flag Formation', type: 'Price Pattern', date: '2023-12-15', result: 'Confirmed', accuracy: 87, impact: '+15.3%' },
  { name: 'Distribution Phase', type: 'Whale Strategy', date: '2023-12-10', result: 'Confirmed', accuracy: 84, impact: '-17.8%' },
  { name: 'Supply Shock', type: 'Token Economics', date: '2023-12-05', result: 'Partial', accuracy: 73, impact: '+10.2%' },
  { name: 'Wash Trading', type: 'Trading Behavior', date: '2023-11-28', result: 'Confirmed', accuracy: 81, impact: '-8.5%' },
  { name: 'Accumulation Phase', type: 'Whale Strategy', date: '2023-11-20', result: 'Failed', accuracy: 79, impact: '-2.3%' },
  { name: 'Pre-announcement Activity', type: 'Insider Trading', date: '2023-11-15', result: 'Confirmed', accuracy: 69, impact: '+41.2%' },
];

export default PatternRecognition;
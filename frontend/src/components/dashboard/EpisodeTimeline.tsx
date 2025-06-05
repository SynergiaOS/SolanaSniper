import React, { useState } from 'react';
import { Clock, Filter, Calendar, ChevronDown, ChevronUp, ArrowRight } from 'lucide-react';

const EpisodeTimeline: React.FC = () => {
  const [expandedEpisode, setExpandedEpisode] = useState<string | null>(null);
  const [filterVisible, setFilterVisible] = useState<boolean>(false);
  
  const toggleEpisode = (id: string) => {
    setExpandedEpisode(expandedEpisode === id ? null : id);
  };
  
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold">Episode Timeline</h1>
        <div className="flex space-x-2">
          <button 
            onClick={() => setFilterVisible(!filterVisible)}
            className={`p-2 rounded-lg ${filterVisible ? 'bg-blue-600' : 'bg-slate-700'} hover:bg-blue-700 transition-colors`}
          >
            <Filter size={20} />
          </button>
          <div className="relative">
            <Calendar className="absolute left-3 top-2.5 text-slate-400" size={16} />
            <select className="pl-9 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-sm">
              <option>Last 24 Hours</option>
              <option>Last 7 Days</option>
              <option>Last 30 Days</option>
              <option>Custom Range</option>
            </select>
          </div>
        </div>
      </div>
      
      {filterVisible && (
        <div className="bg-slate-800/50 rounded-lg p-4 border border-slate-700/50">
          <div className="flex justify-between items-center mb-4">
            <h3 className="font-medium">Filters</h3>
            <button className="text-sm text-blue-500 hover:text-blue-400">
              Reset
            </button>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm text-slate-400 mb-1">Episode Type</label>
              <select className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm">
                <option>All Types</option>
                <option>Trading</option>
                <option>Market Events</option>
                <option>Protocol Actions</option>
                <option>Anomalies</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Confidence Level</label>
              <div className="flex items-center space-x-2">
                <input
                  type="range"
                  min="0"
                  max="100"
                  defaultValue="70"
                  className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer"
                />
                <span className="text-sm">70%+</span>
              </div>
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Entities</label>
              <select className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm">
                <option>All Entities</option>
                <option>Wallets Only</option>
                <option>Tokens Only</option>
                <option>Contracts Only</option>
                <option>Exchanges Only</option>
              </select>
            </div>
          </div>
        </div>
      )}
      
      <div className="relative">
        <div className="absolute left-8 top-0 bottom-0 w-px bg-slate-700/50"></div>
        
        <div className="space-y-6">
          {getEpisodeData().map((episode) => (
            <div key={episode.id} className="relative">
              <div className="flex">
                <div className="absolute left-8 top-8 w-3 h-3 bg-blue-500 rounded-full transform -translate-x-1.5"></div>
                <div className="w-16 text-right pr-4 text-sm text-slate-400 pt-5">
                  {episode.time}
                </div>
                <div className={`flex-1 transition-all duration-300 bg-slate-800/50 rounded-xl border border-slate-700/50 ${
                  expandedEpisode === episode.id ? 'ml-6' : 'ml-4'
                }`}>
                  <div 
                    className="p-4 cursor-pointer hover:bg-slate-700/20 transition-colors"
                    onClick={() => toggleEpisode(episode.id)}
                  >
                    <div className="flex justify-between items-start">
                      <div>
                        <h3 className="font-medium">{episode.title}</h3>
                        <div className="flex items-center text-sm text-slate-400 mt-0.5">
                          <span>{episode.type}</span>
                          <span className="mx-1">•</span>
                          <span>{episode.confidence}% confidence</span>
                        </div>
                      </div>
                      <div className="flex items-center">
                        <div className={`text-xs font-medium px-2 py-1 rounded ${
                          episode.impact === 'High' 
                            ? 'bg-red-500/20 text-red-300' 
                            : episode.impact === 'Medium' 
                              ? 'bg-yellow-500/20 text-yellow-300'
                              : 'bg-blue-500/20 text-blue-300'
                        }`}>
                          {episode.impact} Impact
                        </div>
                        <button className="ml-2">
                          {expandedEpisode === episode.id ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
                        </button>
                      </div>
                    </div>
                    
                    {!expandedEpisode && (
                      <p className="text-sm text-slate-400 mt-1 line-clamp-2">{episode.summary}</p>
                    )}
                  </div>
                  
                  {expandedEpisode === episode.id && (
                    <div className="px-4 pb-4">
                      <div className="bg-slate-700/30 rounded-lg p-4">
                        <p className="text-sm mb-4">{episode.summary}</p>
                        
                        <div className="space-y-4">
                          <div>
                            <h4 className="text-sm font-medium mb-2">Key Entities</h4>
                            <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                              {episode.entities.map((entity, idx) => (
                                <div key={idx} className="flex items-center bg-slate-700/20 p-2 rounded-lg hover:bg-slate-700/30 cursor-pointer">
                                  <div className={`w-6 h-6 rounded-md ${getEntityColorClass(entity.type)} flex items-center justify-center mr-2`}>
                                    <span className="text-xs text-white">{entity.icon}</span>
                                  </div>
                                  <div className="flex-1 min-w-0">
                                    <p className="text-sm truncate">{entity.name}</p>
                                    <p className="text-xs text-slate-400">{entity.type}</p>
                                  </div>
                                </div>
                              ))}
                            </div>
                          </div>
                          
                          <div>
                            <h4 className="text-sm font-medium mb-2">Event Sequence</h4>
                            <div className="relative">
                              <div className="absolute left-3 top-0 bottom-0 w-px bg-slate-700/50"></div>
                              <div className="space-y-3 ml-3 pb-2">
                                {episode.sequence.map((event, idx) => (
                                  <div key={idx} className="relative pl-4">
                                    <div className="absolute left-0 top-1.5 w-2 h-2 rounded-full bg-blue-500 -translate-x-1/2"></div>
                                    <p className="text-sm">{event.action}</p>
                                    <p className="text-xs text-slate-400">{event.time}</p>
                                  </div>
                                ))}
                              </div>
                            </div>
                          </div>
                          
                          {episode.aiInsight && (
                            <div>
                              <h4 className="text-sm font-medium mb-2">AI Insight</h4>
                              <div className="bg-gradient-to-r from-blue-600/10 to-purple-600/10 rounded-lg p-1">
                                <div className="bg-slate-800/90 rounded-lg p-3 border border-blue-500/30">
                                  <div className="flex items-start">
                                    <div className="flex-shrink-0 mt-0.5">
                                      <div className="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center mr-3">
                                        <span className="text-white text-xs font-semibold">AI</span>
                                      </div>
                                    </div>
                                    <div>
                                      <p className="text-sm">{episode.aiInsight}</p>
                                    </div>
                                  </div>
                                </div>
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                      
                      <div className="flex justify-end space-x-2 mt-4">
                        <button className="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm transition-colors">
                          Set Alert
                        </button>
                        <button className="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm transition-colors flex items-center">
                          <span>Investigate</span>
                          <ArrowRight size={14} className="ml-1" />
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
      
      <div className="flex justify-center">
        <button className="px-4 py-2 bg-slate-800 hover:bg-slate-700 rounded-lg text-sm transition-colors">
          Load More
        </button>
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

const getEpisodeData = () => [
  {
    id: 'episode1',
    time: '2h ago',
    title: 'Unusual Token Migration',
    type: 'Anomaly',
    confidence: 92,
    impact: 'High',
    summary: 'Multiple high-value wallets transferring significant SOL amounts to a newly deployed contract, suggesting coordinated migration or potential protocol upgrade.',
    entities: [
      { name: 'SOL', type: 'Token', icon: 'SOL' },
      { name: 'Whale Wallet Alpha', type: 'Wallet', icon: 'WA' },
      { name: 'Unknown Protocol', type: 'Contract', icon: 'UP' },
      { name: 'Institutional Fund', type: 'Wallet', icon: 'IF' },
      { name: 'DEX Router', type: 'Contract', icon: 'DR' },
    ],
    sequence: [
      { action: 'New contract deployed', time: '2h 15m ago' },
      { action: 'Whale Wallet Alpha transferred 25,000 SOL', time: '2h 10m ago' },
      { action: 'Institutional Fund transferred 18,500 SOL', time: '2h 5m ago' },
      { action: 'Contract interaction with DEX Router', time: '1h 50m ago' },
      { action: 'Contract emitted new token event', time: '1h 45m ago' },
    ],
    aiInsight: 'This pattern strongly resembles a coordinated token migration or upgrade. The absence of prior announcements and the involvement of known institutional wallets suggests insider preparation for a major protocol change. Historical data indicates such events often precede significant price movements within 48 hours.',
  },
  {
    id: 'episode2',
    time: '5h ago',
    title: 'DEX Liquidity Drain',
    type: 'Market Event',
    confidence: 85,
    impact: 'Medium',
    summary: 'Significant reduction in liquidity across multiple Solana DEXs for the same token pair, potentially indicating coordinated market positioning.',
    entities: [
      { name: 'USDC', type: 'Token', icon: 'USD' },
      { name: 'Raydium', type: 'Token', icon: 'RAY' },
      { name: 'Decentralized Exchange', type: 'Exchange', icon: 'DE' },
      { name: 'Trader Beta', type: 'Wallet', icon: 'TB' },
      { name: 'Liquidity Pool', type: 'Contract', icon: 'LP' },
    ],
    sequence: [
      { action: 'Liquidity removal from USDC/RAY pool (5.2M)', time: '5h 20m ago' },
      { action: 'Similar withdrawal from secondary DEX', time: '5h 15m ago' },
      { action: 'Price impact increased by 3.2x', time: '5h 10m ago' },
      { action: 'Multiple arbitrage attempts detected', time: '5h 5m ago' },
      { action: 'Price spread between venues reached 4.8%', time: '4h 55m ago' },
    ],
    aiInsight: 'The coordinated liquidity reduction across multiple venues appears strategic rather than panic-driven. The timing and methodical approach suggest possible preparation for a major announcement or market repositioning. Monitor for potential volatility as reduced liquidity amplifies price movements.',
  },
  {
    id: 'episode3',
    time: '12h ago',
    title: 'Governance Proposal Execution',
    type: 'Protocol Action',
    confidence: 96,
    impact: 'Low',
    summary: 'Successful execution of governance proposal SIP-23 affecting protocol parameters and fee structure after passing voting threshold.',
    entities: [
      { name: 'Governance', type: 'Contract', icon: 'GV' },
      { name: 'Lending Protocol', type: 'Contract', icon: 'LP' },
      { name: 'Stake Pool', type: 'Contract', icon: 'SP' },
      { name: 'USDC', type: 'Token', icon: 'USD' },
    ],
    sequence: [
      { action: 'Voting period ended with 76% approval', time: '14h ago' },
      { action: 'Timelock period completed', time: '12h 15m ago' },
      { action: 'Governance execution transaction submitted', time: '12h 10m ago' },
      { action: 'Parameter updates confirmed on-chain', time: '12h 5m ago' },
      { action: 'New fee structure activated', time: '12h ago' },
    ],
    aiInsight: null,
  },
  {
    id: 'episode4',
    time: '1d ago',
    title: 'Potential Insider Trading',
    type: 'Anomaly',
    confidence: 78,
    impact: 'High',
    summary: 'Unusual accumulation of token before major partnership announcement, suggesting possible information asymmetry or insider knowledge.',
    entities: [
      { name: 'Star Atlas', type: 'Token', icon: 'ATL' },
      { name: 'Suspicious Actor', type: 'Wallet', icon: 'SA' },
      { name: 'Centralized Exchange', type: 'Exchange', icon: 'CE' },
      { name: 'OTC Desk', type: 'Exchange', icon: 'OT' },
    ],
    sequence: [
      { action: 'Unusual accumulation began', time: '1d 8h ago' },
      { action: 'Multiple wallets purchased via OTC', time: '1d 5h ago' },
      { action: 'Steady accumulation continued', time: '1d 2h ago' },
      { action: 'Partnership announcement published', time: '23h ago' },
      { action: 'Token price increased by 32%', time: '22h ago' },
    ],
    aiInsight: 'The trading pattern prior to the announcement shows classic signs of information asymmetry. The wallets involved have displayed similar behavior before three previous major announcements, successfully front-running positive news with a 85% correlation pattern. This suggests a potential insider information network.',
  },
  {
    id: 'episode5',
    time: '2d ago',
    title: 'Cross-Chain Bridge Activity',
    type: 'Market Event',
    confidence: 88,
    impact: 'Medium',
    summary: 'Significant increase in assets being bridged from Ethereum to Solana, potentially indicating changing market sentiment or arbitrage opportunities.',
    entities: [
      { name: 'Cross-Chain Bridge', type: 'Contract', icon: 'CB' },
      { name: 'USDC', type: 'Token', icon: 'USD' },
      { name: 'Bridge Contract', type: 'Contract', icon: 'BC' },
      { name: 'Institutional Fund', type: 'Wallet', icon: 'IF' },
    ],
    sequence: [
      { action: 'Bridge inflows increased by 245%', time: '2d 5h ago' },
      { action: 'USDC bridge transfers reached $28M', time: '2d 3h ago' },
      { action: 'Multiple institutional wallets identified', time: '2d 2h ago' },
      { action: 'Liquidity depth increased on Solana DEXs', time: '1d 20h ago' },
      { action: 'Price spread narrowed to 0.2%', time: '1d 15h ago' },
    ],
    aiInsight: 'This significant cross-chain migration appears strategic rather than arbitrage-driven. The transaction patterns and involved entities suggest institutional positioning ahead of potential Solana ecosystem developments. Previous similar migrations have preceded positive price action in the following 5-10 days with 78% correlation.',
  },
];

export default EpisodeTimeline;
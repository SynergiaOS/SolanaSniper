interface GraphNode {
  id: string;
  name: string;
  type: string;
}

interface GraphEdge {
  source: string;
  target: string;
  weight: number;
}

interface NodeDetail {
  id: string;
  name: string;
  type: string;
  created: string;
  lastActive: string;
  riskScore: number;
  metrics?: Record<string, string>;
  connections: Array<{
    name: string;
    type: string;
    relationship: string;
    value: string;
    time: string;
  }>;
  activity: Array<{
    action: string;
    time: string;
  }>;
}

// Mock data for graph visualization
export const getMockGraphData = () => {
  const nodes: GraphNode[] = [
    { id: 'wallet1', name: 'Whale Wallet Alpha', type: 'wallet' },
    { id: 'wallet2', name: 'Trader Beta', type: 'wallet' },
    { id: 'wallet3', name: 'Institutional Fund', type: 'wallet' },
    { id: 'token1', name: 'SOL', type: 'token' },
    { id: 'token2', name: 'USDC', type: 'token' },
    { id: 'token3', name: 'New DeFi Token', type: 'token' },
    { id: 'contract1', name: 'Lending Protocol', type: 'contract' },
    { id: 'contract2', name: 'DEX Router', type: 'contract' },
    { id: 'exchange1', name: 'Centralized Exchange', type: 'exchange' },
    { id: 'exchange2', name: 'Decentralized Exchange', type: 'exchange' },
  ];
  
  const edges: GraphEdge[] = [
    { source: 'wallet1', target: 'token1', weight: 0.9 },
    { source: 'wallet1', target: 'token2', weight: 0.7 },
    { source: 'wallet1', target: 'contract1', weight: 0.5 },
    { source: 'wallet2', target: 'token1', weight: 0.3 },
    { source: 'wallet2', target: 'token3', weight: 0.8 },
    { source: 'wallet2', target: 'exchange2', weight: 0.6 },
    { source: 'wallet3', target: 'token2', weight: 0.9 },
    { source: 'wallet3', target: 'contract2', weight: 0.7 },
    { source: 'token1', target: 'exchange1', weight: 0.4 },
    { source: 'token2', target: 'exchange1', weight: 0.5 },
    { source: 'token3', target: 'exchange2', weight: 0.8 },
    { source: 'contract1', target: 'token2', weight: 0.6 },
    { source: 'contract2', target: 'token1', weight: 0.3 },
    { source: 'contract2', target: 'token3', weight: 0.5 },
  ];
  
  return { nodes, edges };
};

// Mock data for node details panel
export const getMockNodeDetails = (nodeId: string): NodeDetail | null => {
  const mockDetails: Record<string, NodeDetail> = {
    'wallet1': {
      id: 'wallet1',
      name: 'Whale Wallet Alpha',
      type: 'Wallet',
      created: '2023-05-12',
      lastActive: '6 minutes ago',
      riskScore: 2,
      metrics: {
        totalValue: '$8.2M',
        transactions: '142',
        avgSize: '$57.7K',
        holdingPeriod: '95 days',
      },
      connections: [
        { name: 'SOL', type: 'token', relationship: 'Holds', value: '12,450 SOL', time: '4h ago' },
        { name: 'USDC', type: 'token', relationship: 'Holds', value: '3.5M USDC', time: '1d ago' },
        { name: 'Lending Protocol', type: 'contract', relationship: 'Interacts', value: '$1.2M Locked', time: '6m ago' },
      ],
      activity: [
        { action: 'Deposited 500K USDC to Lending Protocol', time: '6m ago' },
        { action: 'Acquired 2,500 SOL', time: '4h ago' },
        { action: 'Withdrew 250K USDC from Exchange', time: '1d ago' },
        { action: 'Provided liquidity to DEX', time: '3d ago' },
      ],
    },
    'token1': {
      id: 'token1',
      name: 'SOL',
      type: 'Token',
      created: '2020-03-16',
      lastActive: 'Continuous',
      riskScore: 1,
      metrics: {
        marketCap: '$32.5B',
        volume24h: '$1.8B',
        holders: '1.2M',
        volatility: 'Medium',
      },
      connections: [
        { name: 'Whale Wallet Alpha', type: 'wallet', relationship: 'Held by', value: '12,450 SOL', time: '4h ago' },
        { name: 'Trader Beta', type: 'wallet', relationship: 'Held by', value: '867 SOL', time: '2d ago' },
        { name: 'Centralized Exchange', type: 'exchange', relationship: 'Listed on', value: '$345M volume', time: 'Today' },
        { name: 'DEX Router', type: 'contract', relationship: 'Traded via', value: '$78M volume', time: 'Today' },
      ],
      activity: [
        { action: 'Price increased by 4.2%', time: '2h ago' },
        { action: 'Large buy order on Exchange', time: '4h ago' },
        { action: 'Protocol upgrade announcement', time: '2d ago' },
        { action: 'New partnership revealed', time: '5d ago' },
      ],
    },
    'contract1': {
      id: 'contract1',
      name: 'Lending Protocol',
      type: 'Contract',
      created: '2022-11-30',
      lastActive: '6 minutes ago',
      riskScore: 3,
      metrics: {
        tvl: '$320M',
        users: '45.2K',
        transactions: '896K',
        avgApY: '8.4%',
      },
      connections: [
        { name: 'Whale Wallet Alpha', type: 'wallet', relationship: 'User', value: '$1.2M Locked', time: '6m ago' },
        { name: 'USDC', type: 'token', relationship: 'Supported', value: '$120M Pool', time: 'Now' },
        { name: 'SOL', type: 'token', relationship: 'Supported', value: '$85M Pool', time: 'Now' },
      ],
      activity: [
        { action: 'Large deposit from Whale Wallet', time: '6m ago' },
        { action: 'Interest rate adjustment', time: '1d ago' },
        { action: 'Protocol upgrade', time: '5d ago' },
        { action: 'Governance proposal passed', time: '2w ago' },
      ],
    },
  };
  
  // Return details if found, or generate a fallback
  if (mockDetails[nodeId]) {
    return mockDetails[nodeId];
  } else {
    // Find the node in our mock graph data
    const allNodes = getMockGraphData().nodes;
    const node = allNodes.find(n => n.id === nodeId);
    
    if (!node) return null;
    
    // Generate generic details
    return {
      id: node.id,
      name: node.name,
      type: node.type,
      created: '2023-01-01',
      lastActive: 'Recently',
      riskScore: Math.floor(Math.random() * 10) + 1,
      connections: [
        { name: 'Connected Entity 1', type: 'wallet', relationship: 'Connected', value: 'Medium', time: 'Recently' },
        { name: 'Connected Entity 2', type: 'token', relationship: 'Connected', value: 'Low', time: 'Last week' },
      ],
      activity: [
        { action: 'Recent activity detected', time: 'Today' },
        { action: 'Connected to new entity', time: 'Last week' },
      ],
    };
  }
};
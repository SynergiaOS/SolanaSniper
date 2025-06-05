import React from 'react';
import { ExternalLink, Copy, X, Activity, DollarSign, Clock } from 'lucide-react';
import { getMockNodeDetails } from '../../utils/mockData';

interface NodeDetailsProps {
  nodeId: string;
  onClose: () => void;
}

const NodeDetails: React.FC<NodeDetailsProps> = ({ nodeId, onClose }) => {
  const nodeDetails = getMockNodeDetails(nodeId);
  
  if (!nodeDetails) {
    return (
      <div className="p-4 text-center">
        <p>Node details not found</p>
        <button onClick={onClose} className="mt-2 px-4 py-2 bg-slate-700 rounded-lg">
          Close
        </button>
      </div>
    );
  }
  
  return (
    <div className="h-full flex flex-col">
      <div className="flex justify-between items-center mb-4">
        <h3 className="font-medium">Node Details</h3>
        <button onClick={onClose} className="p-1 rounded-full hover:bg-slate-700">
          <X size={18} />
        </button>
      </div>
      
      <div className="space-y-6 overflow-y-auto flex-1 pr-1">
        <div className="flex items-center">
          <div className={`w-10 h-10 rounded-full ${getNodeColorClass(nodeDetails.type)} flex items-center justify-center mr-3`}>
            {getNodeIcon(nodeDetails.type)}
          </div>
          <div>
            <h4 className="font-medium">{nodeDetails.name}</h4>
            <div className="flex items-center text-xs text-slate-400">
              <span className="truncate">{nodeDetails.id}</span>
              <button className="ml-1 text-slate-400 hover:text-slate-300">
                <Copy size={12} />
              </button>
            </div>
          </div>
        </div>
        
        <div className="p-3 bg-slate-700/30 rounded-lg">
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div>
              <p className="text-slate-400">Type</p>
              <p>{nodeDetails.type}</p>
            </div>
            <div>
              <p className="text-slate-400">Created</p>
              <p>{nodeDetails.created}</p>
            </div>
            <div>
              <p className="text-slate-400">Last Active</p>
              <p>{nodeDetails.lastActive}</p>
            </div>
            <div>
              <p className="text-slate-400">Risk Score</p>
              <p className={getRiskScoreClass(nodeDetails.riskScore)}>
                {nodeDetails.riskScore}/10
              </p>
            </div>
          </div>
        </div>
        
        {nodeDetails.metrics && (
          <div>
            <h4 className="font-medium text-sm mb-2">Metrics</h4>
            <div className="grid grid-cols-2 gap-3">
              {Object.entries(nodeDetails.metrics).map(([key, value]) => (
                <div key={key} className="bg-slate-700/30 p-3 rounded-lg">
                  <p className="text-xs text-slate-400 mb-1">{formatMetricName(key)}</p>
                  <p className="font-medium">{value}</p>
                </div>
              ))}
            </div>
          </div>
        )}
        
        <div>
          <h4 className="font-medium text-sm mb-2">Connections</h4>
          <div className="space-y-2">
            {nodeDetails.connections.map((connection, index) => (
              <div key={index} className="flex items-center p-2 bg-slate-700/30 rounded-lg hover:bg-slate-700/50 cursor-pointer">
                <div className={`w-8 h-8 rounded-full ${getNodeColorClass(connection.type)} flex items-center justify-center mr-2`}>
                  {getNodeIcon(connection.type)}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="truncate">{connection.name}</p>
                  <p className="text-xs text-slate-400">{connection.relationship}</p>
                </div>
                <div className="text-right">
                  <p className="text-sm">{connection.value}</p>
                  <p className="text-xs text-slate-400">{connection.time}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
        
        <div>
          <h4 className="font-medium text-sm mb-2">Activity</h4>
          <div className="relative">
            <div className="absolute left-3 top-0 bottom-0 w-px bg-slate-700/50"></div>
            <div className="space-y-4 ml-3 pb-2">
              {nodeDetails.activity.map((activity, index) => (
                <div key={index} className="relative pl-4">
                  <div className="absolute left-0 top-1.5 w-2 h-2 rounded-full bg-blue-500 -translate-x-1/2"></div>
                  <p className="text-sm">{activity.action}</p>
                  <p className="text-xs text-slate-400">{activity.time}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
      
      <div className="pt-4 mt-auto border-t border-slate-700/50">
        <div className="flex space-x-2">
          <button className="flex-1 bg-blue-600 hover:bg-blue-700 text-white py-2 rounded-lg text-sm transition-colors">
            Analyze Behavior
          </button>
          <button className="flex items-center justify-center w-10 h-10 bg-slate-700 hover:bg-slate-600 rounded-lg transition-colors">
            <ExternalLink size={18} />
          </button>
        </div>
      </div>
    </div>
  );
};

// Helper functions for styling and formatting
const getNodeColorClass = (type: string): string => {
  switch (type.toLowerCase()) {
    case 'wallet':
      return 'bg-blue-500';
    case 'token':
      return 'bg-purple-500';
    case 'contract':
      return 'bg-green-500';
    case 'exchange':
      return 'bg-yellow-500';
    default:
      return 'bg-gray-500';
  }
};

const getNodeIcon = (type: string) => {
  switch (type.toLowerCase()) {
    case 'wallet':
      return <DollarSign size={16} className="text-white" />;
    case 'token':
      return <Activity size={16} className="text-white" />;
    case 'contract':
      return <div className="text-xs font-bold text-white">SC</div>;
    case 'exchange':
      return <div className="text-xs font-bold text-white">EX</div>;
    default:
      return null;
  }
};

const getRiskScoreClass = (score: number): string => {
  if (score >= 7) return 'text-red-500';
  if (score >= 4) return 'text-yellow-500';
  return 'text-green-500';
};

const formatMetricName = (name: string): string => {
  return name
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, str => str.toUpperCase());
};

export default NodeDetails;
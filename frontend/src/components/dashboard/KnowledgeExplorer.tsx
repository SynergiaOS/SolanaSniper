import React, { useState } from 'react';
import { ZoomIn, ZoomOut, Filter, Search, Download } from 'lucide-react';
import KnowledgeGraph from '../graphs/KnowledgeGraph';
import GraphControls from '../graphs/GraphControls';
import NodeDetails from '../graphs/NodeDetails';

const KnowledgeExplorer: React.FC = () => {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [zoomLevel, setZoomLevel] = useState<number>(1);
  const [filterVisible, setFilterVisible] = useState<boolean>(false);

  const handleNodeSelect = (nodeId: string) => {
    setSelectedNode(nodeId);
  };

  const handleZoomIn = () => {
    setZoomLevel(prev => Math.min(prev + 0.2, 2));
  };

  const handleZoomOut = () => {
    setZoomLevel(prev => Math.max(prev - 0.2, 0.4));
  };

  const toggleFilter = () => {
    setFilterVisible(!filterVisible);
  };

  return (
    <div className="h-full flex flex-col">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-semibold">Knowledge Explorer</h1>
        <div className="flex space-x-2">
          <div className="relative">
            <Search className="absolute left-3 top-2.5 text-slate-400" size={16} />
            <input
              type="text"
              placeholder="Search nodes..."
              className="pl-9 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-sm w-64"
            />
          </div>
          <button 
            onClick={toggleFilter}
            className={`p-2 rounded-lg ${filterVisible ? 'bg-blue-600' : 'bg-slate-700'} hover:bg-blue-700 transition-colors`}
          >
            <Filter size={20} />
          </button>
          <button 
            onClick={handleZoomIn}
            className="p-2 rounded-lg bg-slate-700 hover:bg-slate-600 transition-colors"
          >
            <ZoomIn size={20} />
          </button>
          <button 
            onClick={handleZoomOut}
            className="p-2 rounded-lg bg-slate-700 hover:bg-slate-600 transition-colors"
          >
            <ZoomOut size={20} />
          </button>
          <button className="p-2 rounded-lg bg-slate-700 hover:bg-slate-600 transition-colors">
            <Download size={20} />
          </button>
        </div>
      </div>

      <div className="flex flex-1 space-x-4">
        <div className={`${filterVisible ? 'w-64' : 'w-0'} transition-all duration-300 overflow-hidden`}>
          {filterVisible && (
            <div className="h-full bg-slate-800/50 rounded-xl p-4 border border-slate-700/50">
              <GraphControls />
            </div>
          )}
        </div>
        
        <div className={`flex-1 relative ${selectedNode ? 'lg:w-2/3' : 'w-full'} transition-all duration-300`}>
          <div className="absolute inset-0 bg-slate-800/30 rounded-xl border border-slate-700/50 overflow-hidden">
            <KnowledgeGraph 
              onNodeSelect={handleNodeSelect} 
              zoomLevel={zoomLevel}
            />
          </div>
        </div>
        
        {selectedNode && (
          <div className="w-0 lg:w-1/3 transition-all duration-300 overflow-hidden">
            <div className="h-full bg-slate-800/50 rounded-xl p-4 border border-slate-700/50">
              <NodeDetails nodeId={selectedNode} onClose={() => setSelectedNode(null)} />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default KnowledgeExplorer;
import React, { useEffect, useRef } from 'react';
import { getMockGraphData } from '../../utils/mockData';

interface KnowledgeGraphProps {
  onNodeSelect: (nodeId: string) => void;
  zoomLevel: number;
}

const KnowledgeGraph: React.FC<KnowledgeGraphProps> = ({ onNodeSelect, zoomLevel }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    if (!containerRef.current) return;
    
    // In a real implementation, we would use D3.js to render the graph
    // For this mockup, we'll render a placeholder visualization
    const container = containerRef.current;
    const mockData = getMockGraphData();
    
    // Clear previous content
    container.innerHTML = '';
    
    // Apply zoom level
    const graphContainer = document.createElement('div');
    graphContainer.style.transform = `scale(${zoomLevel})`;
    graphContainer.style.transformOrigin = 'center';
    graphContainer.style.transition = 'transform 0.3s ease';
    graphContainer.style.width = '100%';
    graphContainer.style.height = '100%';
    graphContainer.style.position = 'absolute';
    
    // Create a mock visualization
    const mockNodes = mockData.nodes.map((node, index) => {
      const nodeElement = document.createElement('div');
      nodeElement.className = `absolute rounded-full cursor-pointer transition-all duration-300 
                              border-2 ${getNodeColorClass(node.type)}`;
      nodeElement.style.width = '30px';
      nodeElement.style.height = '30px';
      nodeElement.style.left = `${(Math.sin(index * 0.5) * 40) + 50}%`;
      nodeElement.style.top = `${(Math.cos(index * 0.5) * 40) + 50}%`;
      nodeElement.setAttribute('data-node-id', node.id);
      
      // Add pulse animation for some nodes
      if (index % 3 === 0) {
        const pulse = document.createElement('div');
        pulse.className = `absolute inset-0 rounded-full ${getNodeColorClass(node.type).replace('bg-', 'bg-opacity-30 animate-ping')}`;
        nodeElement.appendChild(pulse);
      }
      
      nodeElement.addEventListener('click', () => onNodeSelect(node.id));
      return nodeElement;
    });
    
    // Add mock edges (lines connecting nodes)
    mockData.edges.forEach(edge => {
      const sourceIndex = mockData.nodes.findIndex(n => n.id === edge.source);
      const targetIndex = mockData.nodes.findIndex(n => n.id === edge.target);
      
      if (sourceIndex >= 0 && targetIndex >= 0) {
        const line = document.createElement('div');
        line.className = 'absolute bg-slate-600/30 transform-gpu';
        
        // Position calculations would be handled by D3 in a real implementation
        // This is just a visual approximation
        const sourceX = (Math.sin(sourceIndex * 0.5) * 40) + 50;
        const sourceY = (Math.cos(sourceIndex * 0.5) * 40) + 50;
        const targetX = (Math.sin(targetIndex * 0.5) * 40) + 50;
        const targetY = (Math.cos(targetIndex * 0.5) * 40) + 50;
        
        const length = Math.sqrt(Math.pow(targetX - sourceX, 2) + Math.pow(targetY - sourceY, 2));
        const angle = Math.atan2(targetY - sourceY, targetX - sourceX) * 180 / Math.PI;
        
        line.style.width = `${length}%`;
        line.style.height = '1px';
        line.style.left = `${sourceX}%`;
        line.style.top = `${sourceY}%`;
        line.style.transformOrigin = '0 0';
        line.style.transform = `rotate(${angle}deg)`;
        
        graphContainer.appendChild(line);
      }
    });
    
    mockNodes.forEach(node => graphContainer.appendChild(node));
    container.appendChild(graphContainer);
    
    return () => {
      // Cleanup
      container.innerHTML = '';
    };
  }, [onNodeSelect, zoomLevel]);
  
  // Helper function to determine node color based on type
  const getNodeColorClass = (type: string): string => {
    switch (type) {
      case 'wallet':
        return 'bg-blue-500 border-blue-300';
      case 'token':
        return 'bg-purple-500 border-purple-300';
      case 'contract':
        return 'bg-green-500 border-green-300';
      case 'exchange':
        return 'bg-yellow-500 border-yellow-300';
      default:
        return 'bg-gray-500 border-gray-300';
    }
  };
  
  return (
    <div 
      ref={containerRef} 
      className="w-full h-full relative overflow-hidden"
      style={{ cursor: 'grab' }}
    >
      <div className="absolute inset-0 flex items-center justify-center">
        <div className="text-slate-500">Loading graph visualization...</div>
      </div>
    </div>
  );
};

export default KnowledgeGraph;
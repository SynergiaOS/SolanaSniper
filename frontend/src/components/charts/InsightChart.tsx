import React, { useEffect, useRef } from 'react';

interface InsightChartProps {
  type: 'accuracy' | 'volume' | 'anomaly';
}

const InsightChart: React.FC<InsightChartProps> = ({ type }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  
  useEffect(() => {
    if (!canvasRef.current) return;
    
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    
    if (!ctx) return;
    
    // Set canvas dimensions with device pixel ratio for sharp rendering
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    
    // Reset canvas styles
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;
    
    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Render different chart types
    switch (type) {
      case 'accuracy':
        renderAccuracyChart(ctx, rect.width, rect.height);
        break;
      case 'volume':
        renderVolumeChart(ctx, rect.width, rect.height);
        break;
      case 'anomaly':
        renderAnomalyChart(ctx, rect.width, rect.height);
        break;
    }
  }, [type]);
  
  const renderAccuracyChart = (ctx: CanvasRenderingContext2D, width: number, height: number) => {
    // Generate mock data
    const data = Array(24).fill(0).map((_, i) => {
      return {
        x: i,
        y: 70 + Math.random() * 20 + (i > 12 ? 5 : 0), // Trending slightly up in second half
      };
    });
    
    // Draw grid
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.1)';
    ctx.lineWidth = 1;
    
    // Horizontal grid lines
    for (let y = 0; y < height; y += height / 5) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
    
    // Vertical grid lines
    for (let x = 0; x < width; x += width / 12) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
    
    // X-axis labels
    ctx.fillStyle = 'rgba(148, 163, 184, 0.7)';
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'center';
    
    for (let i = 0; i < 7; i++) {
      const x = (width / 6) * i;
      ctx.fillText(`${i * 4}h`, x, height - 5);
    }
    
    // Create gradient for line
    const gradient = ctx.createLinearGradient(0, 0, 0, height);
    gradient.addColorStop(0, 'rgba(59, 130, 246, 1)');
    gradient.addColorStop(1, 'rgba(59, 130, 246, 0)');
    
    // Draw line
    ctx.strokeStyle = 'rgba(59, 130, 246, 1)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    
    // Scale data to fit canvas
    const scaleX = width / (data.length - 1);
    const scaleY = height / 40; // Scale factor for y values
    const offsetY = height - 50 * scaleY; // Offset to position the line
    
    data.forEach((point, i) => {
      const x = i * scaleX;
      const y = offsetY - (point.y - 70) * scaleY;
      
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });
    
    ctx.stroke();
    
    // Draw area under the line
    ctx.lineTo(width, height);
    ctx.lineTo(0, height);
    ctx.closePath();
    ctx.fillStyle = 'rgba(59, 130, 246, 0.1)';
    ctx.fill();
    
    // Draw data points
    ctx.fillStyle = 'rgba(59, 130, 246, 1)';
    data.forEach((point, i) => {
      if (i % 4 === 0) { // Draw fewer points for cleaner look
        const x = i * scaleX;
        const y = offsetY - (point.y - 70) * scaleY;
        
        ctx.beginPath();
        ctx.arc(x, y, 3, 0, Math.PI * 2);
        ctx.fill();
      }
    });
  };
  
  const renderVolumeChart = (ctx: CanvasRenderingContext2D, width: number, height: number) => {
    // This would implement volume chart visualization
    // For brevity, not implementing all chart types
  };
  
  const renderAnomalyChart = (ctx: CanvasRenderingContext2D, width: number, height: number) => {
    // This would implement anomaly chart visualization
    // For brevity, not implementing all chart types
  };
  
  return (
    <canvas 
      ref={canvasRef} 
      className="w-full h-full"
    />
  );
};

export default InsightChart;
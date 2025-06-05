import React from 'react';
import { TrendingUp, TrendingDown } from 'lucide-react';

interface MetricCardProps {
  title: string;
  value: string;
  change: string;
  isPositive: boolean;
  icon: React.ReactNode;
}

const MetricCard: React.FC<MetricCardProps> = ({ title, value, change, isPositive, icon }) => {
  return (
    <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700/50 hover:border-slate-600/80 transition-colors">
      <div className="flex justify-between items-start mb-2">
        <p className="text-sm text-slate-400">{title}</p>
        {icon}
      </div>
      <p className="text-2xl font-semibold mb-2">{value}</p>
      <div className={`flex items-center text-sm ${isPositive ? 'text-green-500' : 'text-red-500'}`}>
        {isPositive ? <TrendingUp size={16} className="mr-1" /> : <TrendingDown size={16} className="mr-1" />}
        <span>{change}</span>
      </div>
    </div>
  );
};

export default MetricCard;
import React from 'react';
import { Activity, TrendingUp, DollarSign, Zap, BarChart3, Settings } from 'lucide-react';

interface SidebarProps {
  isOpen: boolean;
  activeSection: string;
  onSectionChange: (section: any) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ isOpen, activeSection, onSectionChange }) => {
  const navigationItems = [
    { id: 'status', name: 'Bot Status', icon: Activity },
    { id: 'signals', name: 'Live Signals', icon: Zap },
    { id: 'trades', name: 'Trade History', icon: DollarSign },
    { id: 'performance', name: 'Performance', icon: TrendingUp },
    { id: 'analytics', name: 'Analytics', icon: BarChart3 },
    { id: 'settings', name: 'Settings', icon: Settings },
  ];

  return (
    <aside
      className={`fixed inset-y-0 left-0 z-20 flex flex-col bg-finance-900 border-r border-finance-700 transition-all duration-300 ${
        isOpen ? 'w-64' : 'w-20'
      }`}
    >
      <div className="flex items-center justify-between h-16 px-4 border-b border-finance-700">
        <div className={`flex items-center ${isOpen ? 'justify-start' : 'justify-center w-full'}`}>
          <div className="h-8 w-8 bg-gradient-to-r from-trading-bull-primary to-status-info rounded-md flex items-center justify-center text-white font-bold shadow-trading">
            S
          </div>
          <h1 className={`ml-2 font-semibold tracking-tight text-lg text-white transition-opacity duration-300 ${isOpen ? 'opacity-100' : 'opacity-0 hidden'}`}>
            SniperBot
          </h1>
        </div>
      </div>
      <nav className="flex-1 pt-4 pb-4 overflow-y-auto">
        <ul className="space-y-1 px-2">
          {navigationItems.map((item) => {
            const IconComponent = item.icon;
            return (
              <li key={item.id}>
                <button
                  onClick={() => onSectionChange(item.id)}
                  className={`flex items-center w-full p-2 rounded-lg transition-colors ${
                    activeSection === item.id
                      ? 'bg-status-info text-white shadow-trading'
                      : 'text-finance-300 hover:bg-finance-800 hover:text-white'
                  } ${!isOpen ? 'justify-center' : ''}`}
                >
                  <IconComponent size={20} />
                  <span className={`ml-3 transition-opacity duration-300 ${isOpen ? 'opacity-100' : 'opacity-0 hidden'}`}>
                    {item.name}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      </nav>
      <div className="p-4 border-t border-finance-700">
        <div className={`flex items-center ${isOpen ? 'justify-between' : 'justify-center'}`}>
          <div className={`transition-opacity duration-300 ${isOpen ? 'opacity-100' : 'opacity-0 hidden'}`}>
            <p className="text-sm text-finance-400">SniperBot v2.0</p>
          </div>
        </div>
      </div>
    </aside>
  );
};

export default Sidebar;
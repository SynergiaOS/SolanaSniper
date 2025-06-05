import React from 'react';
import { Menu, Bell, Search, Sun, Moon } from 'lucide-react';
import { useTheme } from '../../context/ThemeContext';

interface HeaderProps {
  toggleSidebar: () => void;
  sidebarOpen: boolean;
}

const Header: React.FC<HeaderProps> = ({ toggleSidebar, sidebarOpen }) => {
  const { theme, toggleTheme } = useTheme();

  return (
    <header className="sticky top-0 z-10 bg-slate-800/80 backdrop-blur-sm dark:bg-slate-800/80 light:bg-white/80 border-b border-slate-700/50 light:border-slate-200/50">
      <div className="flex items-center justify-between p-4">
        <div className="flex items-center">
          <button 
            onClick={toggleSidebar}
            className="p-2 mr-4 rounded-lg hover:bg-slate-700/50 light:hover:bg-slate-200/50 transition-colors"
            aria-label={sidebarOpen ? "Close sidebar" : "Open sidebar"}
          >
            <Menu size={24} />
          </button>
          <div className="relative hidden md:block">
            <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
              <Search size={18} className="text-slate-400" />
            </div>
            <input 
              type="search" 
              className="block w-full p-2 pl-10 text-sm rounded-lg bg-slate-700/50 border border-slate-600/50 light:bg-slate-100 light:border-slate-200 focus:ring-blue-500 focus:border-blue-500" 
              placeholder="Search transactions, wallets, tokens..." 
            />
          </div>
        </div>
        <div className="flex items-center space-x-4">
          <button 
            onClick={toggleTheme} 
            className="p-2 rounded-lg hover:bg-slate-700/50 light:hover:bg-slate-200/50 transition-colors"
            aria-label={theme === 'dark' ? "Switch to light mode" : "Switch to dark mode"}
          >
            {theme === 'dark' ? <Sun size={20} /> : <Moon size={20} />}
          </button>
          <button 
            className="p-2 rounded-lg hover:bg-slate-700/50 light:hover:bg-slate-200/50 transition-colors relative"
            aria-label="Notifications"
          >
            <Bell size={20} />
            <span className="absolute top-1 right-1 w-2 h-2 bg-blue-500 rounded-full"></span>
          </button>
          <div className="h-8 w-8 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full"></div>
        </div>
      </div>
    </header>
  );
};

export default Header;
import React, { useState } from 'react';
import Header from './Header';
import Sidebar from './Sidebar';
import BotStatusDashboard from '../dashboard/BotStatusDashboard';
import SignalsDashboard from '../dashboard/SignalsDashboard';
import TradesDashboard from '../dashboard/TradesDashboard';
import ActivePositions from '../ActivePositions';
import LiveEventLog from '../LiveEventLog';
import PortfolioOverview from '../PortfolioOverview';

type DashboardSection = 'status' | 'portfolio' | 'signals' | 'trades' | 'positions' | 'events' | 'performance' | 'analytics' | 'settings';

const DashboardLayout: React.FC = () => {
  const [activeSection, setActiveSection] = useState<DashboardSection>('status');
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const toggleSidebar = () => setSidebarOpen(!sidebarOpen);

  const renderSection = () => {
    switch (activeSection) {
      case 'status':
        return <BotStatusDashboard />;
      case 'portfolio':
        return <PortfolioOverview />;
      case 'signals':
        return <SignalsDashboard />;
      case 'trades':
        return <TradesDashboard />;
      case 'positions':
        return <ActivePositions />;
      case 'events':
        return <LiveEventLog />;
      case 'performance':
        return <div className="p-8 text-center text-slate-500">Performance analytics coming soon...</div>;
      case 'analytics':
        return <div className="p-8 text-center text-slate-500">Advanced analytics coming soon...</div>;
      case 'settings':
        return <div className="p-8 text-center text-slate-500">Settings panel coming soon...</div>;
      default:
        return <BotStatusDashboard />;
    }
  };

  return (
    <div className="flex h-screen bg-finance-50 text-finance-900">
      <Sidebar 
        isOpen={sidebarOpen} 
        activeSection={activeSection} 
        onSectionChange={setActiveSection} 
      />
      <div className={`flex flex-col flex-1 transition-all duration-300 ${sidebarOpen ? 'md:ml-64' : 'md:ml-20'}`}>
        <Header toggleSidebar={toggleSidebar} sidebarOpen={sidebarOpen} />
        <main className="flex-1 overflow-y-auto p-4 md:p-6">
          {renderSection()}
        </main>
      </div>
    </div>
  );
};

export default DashboardLayout;
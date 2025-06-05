import React from 'react';
import { ThemeProvider } from './context/ThemeContext';
import { SniperBotProvider } from './context/SniperBotContext';
import DashboardLayout from './components/layout/DashboardLayout';

function App() {
  return (
    <ThemeProvider>
      <SniperBotProvider>
        <DashboardLayout />
      </SniperBotProvider>
    </ThemeProvider>
  );
}

export default App;
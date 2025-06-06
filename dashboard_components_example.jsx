// === REACT COMPONENTS FOR SNIPERBOT DASHBOARD ===
// Modern, professional trading dashboard components

import { useState, useEffect } from 'react';
import useSWR from 'swr';

// === 1. Main Dashboard Component ===
const fetcher = (url) => fetch(url).then((res) => res.json());

export default function SniperBotDashboard() {
  // Auto-refresh every 3 seconds
  const { data: stats, error: statsError } = useSWR('/api/dashboard/stats', fetcher, {
    refreshInterval: 3000,
  });
  
  const { data: realtime, error: realtimeError } = useSWR('/api/dashboard/realtime', fetcher, {
    refreshInterval: 1000, // More frequent for realtime data
  });
  
  const { data: activities, error: activitiesError } = useSWR('/api/dashboard/activity?limit=10', fetcher, {
    refreshInterval: 5000,
  });
  
  const { data: opportunities, error: opportunitiesError } = useSWR('/api/dashboard/opportunities', fetcher, {
    refreshInterval: 10000,
  });

  if (statsError || realtimeError) {
    return <div className="error">Failed to load dashboard data</div>;
  }

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <h1>🧠 SniperBot 2.0 - The Persistent Brain</h1>
        <div className="status-indicator">
          <span className={`status-dot ${stats?.bot_status === 'Running' ? 'running' : 'stopped'}`}></span>
          <span>{stats?.bot_status || 'Unknown'}</span>
        </div>
      </header>

      <div className="dashboard-grid">
        <StatsCards stats={stats} />
        <RealtimeMetrics realtime={realtime} />
        <ActivityFeed activities={activities} />
        <OpportunitiesTable opportunities={opportunities} />
      </div>
    </div>
  );
}

// === 2. Stats Cards Component ===
function StatsCards({ stats }) {
  if (!stats) return <div className="loading">Loading stats...</div>;

  const formatUptime = (seconds) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount);
  };

  return (
    <div className="stats-grid">
      <div className="stat-card">
        <div className="stat-value">{stats.total_opportunities.toLocaleString()}</div>
        <div className="stat-label">Total Opportunities</div>
      </div>
      
      <div className="stat-card">
        <div className="stat-value">{stats.active_opportunities}</div>
        <div className="stat-label">Active Opportunities</div>
      </div>
      
      <div className="stat-card">
        <div className="stat-value">{stats.total_trades}</div>
        <div className="stat-label">Total Trades</div>
      </div>
      
      <div className="stat-card">
        <div className="stat-value">{formatCurrency(stats.total_pnl_usd)}</div>
        <div className="stat-label">Total P&L</div>
      </div>
      
      <div className="stat-card">
        <div className="stat-value">{stats.success_rate.toFixed(1)}%</div>
        <div className="stat-label">Success Rate</div>
      </div>
      
      <div className="stat-card">
        <div className="stat-value">{formatUptime(stats.uptime_seconds)}</div>
        <div className="stat-label">Uptime</div>
      </div>
    </div>
  );
}

// === 3. Realtime Metrics Component ===
function RealtimeMetrics({ realtime }) {
  if (!realtime) return <div className="loading">Loading realtime data...</div>;

  return (
    <div className="realtime-panel">
      <h3>⚡ Real-time Metrics</h3>
      
      <div className="metrics-grid">
        <div className="metric">
          <span className="metric-label">Cycle #</span>
          <span className="metric-value">{realtime.cycle_number}</span>
        </div>
        
        <div className="metric">
          <span className="metric-label">Cycle Duration</span>
          <span className="metric-value">{realtime.cycle_duration_ms}ms</span>
        </div>
        
        <div className="metric">
          <span className="metric-label">Processed</span>
          <span className="metric-value">{realtime.opportunities_processed}</span>
        </div>
        
        <div className="metric">
          <span className="metric-label">DB Status</span>
          <span className={`metric-value ${realtime.db_connected ? 'connected' : 'disconnected'}`}>
            {realtime.db_connected ? '🟢 Connected' : '🔴 Disconnected'}
          </span>
        </div>
      </div>
      
      <div className="last-update">
        Last update: {new Date(realtime.timestamp).toLocaleTimeString()}
      </div>
    </div>
  );
}

// === 4. Activity Feed Component ===
function ActivityFeed({ activities }) {
  if (!activities) return <div className="loading">Loading activity...</div>;

  const getSeverityIcon = (severity) => {
    switch (severity) {
      case 'Error': return '🔴';
      case 'Warning': return '🟡';
      case 'Info': return '🔵';
      default: return '⚪';
    }
  };

  return (
    <div className="activity-panel">
      <h3>📋 Recent Activity</h3>
      
      <div className="activity-list">
        {activities.map((activity, index) => (
          <div key={activity.id || index} className="activity-item">
            <div className="activity-icon">
              {getSeverityIcon(activity.severity)}
            </div>
            <div className="activity-content">
              <div className="activity-description">{activity.description}</div>
              <div className="activity-time">
                {new Date(activity.timestamp).toLocaleString()}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// === 5. Opportunities Table Component ===
function OpportunitiesTable({ opportunities }) {
  if (!opportunities) return <div className="loading">Loading opportunities...</div>;

  const formatLiquidity = (liquidity) => {
    if (liquidity >= 1000000) {
      return `$${(liquidity / 1000000).toFixed(1)}M`;
    } else if (liquidity >= 1000) {
      return `$${(liquidity / 1000).toFixed(1)}K`;
    }
    return `$${liquidity.toFixed(0)}`;
  };

  return (
    <div className="opportunities-panel">
      <h3>🎯 Recent Opportunities</h3>
      
      <div className="table-container">
        <table className="opportunities-table">
          <thead>
            <tr>
              <th>Token</th>
              <th>Liquidity</th>
              <th>Age</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {opportunities.slice(0, 10).map((opp, index) => (
              <tr key={opp.key || index}>
                <td>
                  <div className="token-info">
                    <div className="token-address">
                      {opp.candidate?.address?.slice(0, 8)}...
                    </div>
                  </div>
                </td>
                <td>{formatLiquidity(opp.candidate?.liquidity_usd || 0)}</td>
                <td>
                  {opp.discovered_at ? 
                    Math.floor((Date.now() - new Date(opp.discovered_at).getTime()) / 60000) + 'm' 
                    : 'Unknown'
                  }
                </td>
                <td>
                  <span className={`status-badge ${opp.status?.toLowerCase() || 'unknown'}`}>
                    {opp.status || 'Unknown'}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// === CSS STYLES (dashboard.module.css) ===
/*
.dashboard {
  min-height: 100vh;
  background: linear-gradient(135deg, #0f0f23 0%, #1a1a2e 100%);
  color: #ffffff;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2rem;
  border-bottom: 1px solid #333;
}

.dashboard-header h1 {
  font-size: 2rem;
  font-weight: 700;
  margin: 0;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 500;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #666;
}

.status-dot.running {
  background: #00ff88;
  box-shadow: 0 0 10px #00ff88;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: auto auto auto;
  gap: 2rem;
  padding: 2rem;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  grid-column: 1 / -1;
}

.stat-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.5rem;
  text-align: center;
}

.stat-value {
  font-size: 2rem;
  font-weight: 700;
  color: #00ff88;
  margin-bottom: 0.5rem;
}

.stat-label {
  font-size: 0.875rem;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.realtime-panel, .activity-panel, .opportunities-panel {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.5rem;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: #888;
}

.error {
  color: #ff4444;
  text-align: center;
  padding: 2rem;
}
*/

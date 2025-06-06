// === NEXT.JS API ROUTES FOR SNIPERBOT DASHBOARD ===
// Place these files in your Next.js project under pages/api/ or app/api/

// === 1. pages/api/dashboard/stats.js ===
import { createClient } from 'redis';

export default async function handler(req, res) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const client = createClient({
    url: process.env.DRAGONFLY_URL
  });

  try {
    await client.connect();
    
    // Get dashboard statistics
    const statsData = await client.get('dashboard:stats');
    
    if (statsData) {
      const stats = JSON.parse(statsData);
      res.status(200).json(stats);
    } else {
      // Return default stats if none exist
      res.status(200).json({
        total_opportunities: 0,
        active_opportunities: 0,
        total_trades: 0,
        active_positions: 0,
        total_pnl_usd: 0.0,
        success_rate: 0.0,
        uptime_seconds: 0,
        last_updated: new Date().toISOString(),
        bot_status: "Unknown",
        processing_speed: 0.0
      });
    }
    
  } catch (error) {
    console.error('Dashboard stats error:', error);
    res.status(500).json({ error: 'Failed to fetch dashboard stats' });
  } finally {
    await client.quit();
  }
}

// === 2. pages/api/dashboard/realtime.js ===
import { createClient } from 'redis';

export default async function handler(req, res) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const client = createClient({
    url: process.env.DRAGONFLY_URL
  });

  try {
    await client.connect();
    
    // Get realtime metrics
    const metricsData = await client.get('realtime:metrics');
    
    if (metricsData) {
      const metrics = JSON.parse(metricsData);
      res.status(200).json(metrics);
    } else {
      res.status(200).json({
        cycle_number: 0,
        cycle_duration_ms: 0,
        opportunities_processed: 0,
        decisions_made: 0,
        timestamp: new Date().toISOString(),
        memory_usage_mb: 0.0,
        cpu_usage_percent: 0.0,
        db_connected: false
      });
    }
    
  } catch (error) {
    console.error('Realtime metrics error:', error);
    res.status(500).json({ error: 'Failed to fetch realtime metrics' });
  } finally {
    await client.quit();
  }
}

// === 3. pages/api/dashboard/activity.js ===
import { createClient } from 'redis';

export default async function handler(req, res) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { limit = 20 } = req.query;
  
  const client = createClient({
    url: process.env.DRAGONFLY_URL
  });

  try {
    await client.connect();
    
    // Get recent activity events
    const activityData = await client.lRange('dashboard:activity_feed', 0, parseInt(limit) - 1);
    
    const activities = activityData.map(data => {
      try {
        return JSON.parse(data);
      } catch (e) {
        console.error('Failed to parse activity data:', e);
        return null;
      }
    }).filter(Boolean);
    
    res.status(200).json(activities);
    
  } catch (error) {
    console.error('Activity feed error:', error);
    res.status(500).json({ error: 'Failed to fetch activity feed' });
  } finally {
    await client.quit();
  }
}

// === 4. pages/api/dashboard/opportunities.js ===
import { createClient } from 'redis';

export default async function handler(req, res) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const client = createClient({
    url: process.env.DRAGONFLY_URL
  });

  try {
    await client.connect();
    
    // Get list of raw opportunity keys
    const opportunityKeys = await client.lRange('all_raw_opportunities', 0, 49); // Last 50
    
    let opportunities = [];
    
    if (opportunityKeys.length > 0) {
      // Get data for each opportunity
      const multi = client.multi();
      opportunityKeys.forEach(key => multi.get(key));
      const results = await multi.exec();
      
      opportunities = results.map((result, index) => {
        if (result && result.length > 1 && result[1]) {
          try {
            const data = JSON.parse(result[1]);
            return {
              key: opportunityKeys[index],
              ...data
            };
          } catch (e) {
            console.error('Failed to parse opportunity data:', e);
            return null;
          }
        }
        return null;
      }).filter(Boolean);
    }
    
    res.status(200).json(opportunities);
    
  } catch (error) {
    console.error('Opportunities error:', error);
    res.status(500).json({ error: 'Failed to fetch opportunities' });
  } finally {
    await client.quit();
  }
}

// === 5. pages/api/dashboard/bot-status.js ===
import { createClient } from 'redis';

export default async function handler(req, res) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const client = createClient({
    url: process.env.DRAGONFLY_URL
  });

  try {
    await client.connect();
    
    // Get bot status
    const statusData = await client.get('bot:status');
    
    if (statusData) {
      const status = JSON.parse(statusData);
      res.status(200).json(status);
    } else {
      res.status(200).json({
        state: "Unknown",
        mode: "Unknown",
        started_at: new Date().toISOString(),
        last_activity: new Date().toISOString(),
        config_hash: "unknown",
        version: "unknown",
        health: { status: "unknown" }
      });
    }
    
  } catch (error) {
    console.error('Bot status error:', error);
    res.status(500).json({ error: 'Failed to fetch bot status' });
  } finally {
    await client.quit();
  }
}

// === ENVIRONMENT VARIABLES (.env.local) ===
/*
DRAGONFLY_URL=redis://localhost:6379
*/

// === PACKAGE.JSON DEPENDENCIES ===
/*
{
  "dependencies": {
    "redis": "^4.6.0",
    "next": "^14.0.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0"
  }
}
*/

import React, { useState, useEffect, useRef } from 'react';
import { 
  Brain, 
  Target, 
  AlertTriangle, 
  Info, 
  Bug, 
  Zap,
  Filter,
  Pause,
  Play
} from 'lucide-react';

interface LiveEvent {
  id: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warning' | 'error' | 'critical';
  component: string;
  event_type: string;
  message: string;
  details?: any;
}

const LiveEventLog: React.FC = () => {
  const [events, setEvents] = useState<LiveEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isPaused, setIsPaused] = useState(false);
  const [selectedLevel, setSelectedLevel] = useState<string>('all');
  const [selectedComponent, setSelectedComponent] = useState<string>('all');
  const logContainerRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  const fetchEvents = async () => {
    if (isPaused) return;

    try {
      const params = new URLSearchParams();
      params.append('limit', '50');
      if (selectedLevel !== 'all') {
        params.append('level', selectedLevel);
      }
      if (selectedComponent !== 'all') {
        params.append('component', selectedComponent);
      }

      const response = await fetch(`http://localhost:8084/api/live-events?${params}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch events: ${response.statusText}`);
      }
      const data = await response.json();
      setEvents(data);
      setError(null);

      // Auto-scroll to bottom if enabled
      if (autoScroll && logContainerRef.current) {
        logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchEvents();
    const interval = setInterval(fetchEvents, 2000); // Refresh every 2 seconds
    return () => clearInterval(interval);
  }, [isPaused, selectedLevel, selectedComponent]);

  const getEventIcon = (level: string, component: string) => {
    if (component.toLowerCase().includes('brain') || component.toLowerCase().includes('ai')) {
      return <Brain size={16} className="text-purple-400" />;
    }
    if (component.toLowerCase().includes('reflex') || component.toLowerCase().includes('core')) {
      return <Zap size={16} className="text-yellow-400" />;
    }
    if (component.toLowerCase().includes('decision') || component.toLowerCase().includes('engine')) {
      return <Target size={16} className="text-blue-400" />;
    }

    switch (level) {
      case 'critical':
      case 'error':
        return <AlertTriangle size={16} className="text-red-400" />;
      case 'warning':
        return <AlertTriangle size={16} className="text-yellow-400" />;
      case 'debug':
        return <Bug size={16} className="text-gray-400" />;
      default:
        return <Info size={16} className="text-blue-400" />;
    }
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'critical':
        return 'text-red-500 bg-red-900/20';
      case 'error':
        return 'text-red-400 bg-red-900/10';
      case 'warning':
        return 'text-yellow-400 bg-yellow-900/10';
      case 'debug':
        return 'text-gray-400 bg-gray-900/10';
      default:
        return 'text-blue-400 bg-blue-900/10';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3,
    });
  };

  const getUniqueComponents = () => {
    const components = new Set(events.map(e => e.component));
    return Array.from(components).sort();
  };

  if (loading && events.length === 0) {
    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">Live Event Log</h2>
        <div className="text-gray-400">Loading events...</div>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6 h-96 flex flex-col">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold text-white">
          Live Event Log ({events.length})
        </h2>
        <div className="flex items-center gap-2">
          {/* Filters */}
          <select
            value={selectedLevel}
            onChange={(e) => setSelectedLevel(e.target.value)}
            className="px-2 py-1 bg-gray-700 text-white rounded text-sm"
          >
            <option value="all">All Levels</option>
            <option value="critical">Critical</option>
            <option value="error">Error</option>
            <option value="warning">Warning</option>
            <option value="info">Info</option>
            <option value="debug">Debug</option>
          </select>

          <select
            value={selectedComponent}
            onChange={(e) => setSelectedComponent(e.target.value)}
            className="px-2 py-1 bg-gray-700 text-white rounded text-sm"
          >
            <option value="all">All Components</option>
            {getUniqueComponents().map(component => (
              <option key={component} value={component}>{component}</option>
            ))}
          </select>

          {/* Controls */}
          <button
            onClick={() => setIsPaused(!isPaused)}
            className={`px-3 py-1 rounded text-sm flex items-center gap-1 ${
              isPaused ? 'bg-green-600 hover:bg-green-700' : 'bg-yellow-600 hover:bg-yellow-700'
            } text-white transition-colors`}
          >
            {isPaused ? <Play size={14} /> : <Pause size={14} />}
            {isPaused ? 'Resume' : 'Pause'}
          </button>

          <button
            onClick={() => setAutoScroll(!autoScroll)}
            className={`px-3 py-1 rounded text-sm ${
              autoScroll ? 'bg-blue-600 hover:bg-blue-700' : 'bg-gray-600 hover:bg-gray-700'
            } text-white transition-colors`}
          >
            Auto-scroll: {autoScroll ? 'ON' : 'OFF'}
          </button>
        </div>
      </div>

      {error && (
        <div className="text-red-400 mb-2 text-sm">Error: {error}</div>
      )}

      <div 
        ref={logContainerRef}
        className="flex-1 overflow-y-auto bg-gray-900 rounded p-3 font-mono text-sm space-y-1"
      >
        {events.length === 0 ? (
          <div className="text-gray-400 text-center py-8">
            No events to display
          </div>
        ) : (
          events.map((event) => (
            <div
              key={event.id}
              className={`flex items-start gap-2 p-2 rounded ${getLevelColor(event.level)}`}
            >
              <div className="flex-shrink-0 mt-0.5">
                {getEventIcon(event.level, event.component)}
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 text-xs text-gray-400 mb-1">
                  <span>{formatTimestamp(event.timestamp)}</span>
                  <span className="px-1 py-0.5 bg-gray-700 rounded text-xs">
                    {event.component}
                  </span>
                  <span className="px-1 py-0.5 bg-gray-600 rounded text-xs">
                    {event.event_type}
                  </span>
                </div>
                <div className="text-white text-sm break-words">
                  {event.message}
                </div>
                {event.details && (
                  <details className="mt-1">
                    <summary className="text-xs text-gray-400 cursor-pointer hover:text-gray-300">
                      Details
                    </summary>
                    <pre className="text-xs text-gray-300 mt-1 bg-gray-800 p-2 rounded overflow-x-auto">
                      {JSON.stringify(event.details, null, 2)}
                    </pre>
                  </details>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default LiveEventLog;

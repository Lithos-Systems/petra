import React, { useState, useEffect, useMemo } from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Bell, AlertCircle, AlertTriangle, Info, CheckCircle } from 'lucide-react';

// ISA-18.2 compliant alarm display colors
const ALARM_COLORS = {
  critical: {
    background: '#FF0000',
    text: '#FFFFFF',
    icon: AlertCircle
  },
  high: {
    background: '#FF8C00', 
    text: '#FFFFFF',
    icon: AlertTriangle
  },
  medium: {
    background: '#FFFF00',
    text: '#000000', 
    icon: AlertTriangle
  },
  low: {
    background: '#00FFFF',
    text: '#000000',
    icon: Info
  },
  normal: {
    background: '#F0F0F0',
    text: '#000000',
    icon: CheckCircle
  }
};

// Alarm states per ISA-18.2
const ALARM_STATES = {
  NORMAL: 'Normal',
  UNACKNOWLEDGED: 'Unacknowledged',
  ACKNOWLEDGED: 'Acknowledged', 
  RTN_UNACK: 'RTN-Unack',
  SUPPRESSED: 'Suppressed',
  OUT_OF_SERVICE: 'Out of Service',
  SHELVED: 'Shelved'
};

// Mock alarm data - replace with actual signal bus connection
const generateMockAlarms = () => [
  {
    name: 'PT_101_HH',
    description: 'Reactor Pressure High-High',
    tagName: 'PT-101',
    priority: 'critical',
    state: ALARM_STATES.UNACKNOWLEDGED,
    value: 265.3,
    units: 'PSIG',
    setpoint: 250.0,
    area: 'Reactor Area',
    equipment: 'R-101',
    activationTime: new Date(Date.now() - 120000),
    consequence: 'Reactor rupture risk',
    correctiveAction: 'Open PV-101 immediately'
  },
  {
    name: 'LT_201_H',
    description: 'Feed Tank Level High',
    tagName: 'LT-201',
    priority: 'high',
    state: ALARM_STATES.ACKNOWLEDGED,
    value: 92.5,
    units: '%',
    setpoint: 90.0,
    area: 'Tank Farm',
    equipment: 'TK-201',
    activationTime: new Date(Date.now() - 300000),
    acknowledgedBy: 'J. Smith',
    acknowledgedAt: new Date(Date.now() - 180000)
  },
  {
    name: 'AT_401_H',
    description: 'Product pH High',
    tagName: 'AT-401',
    priority: 'medium',
    state: ALARM_STATES.RTN_UNACK,
    value: 7.8,
    units: 'pH',
    setpoint: 8.5,
    area: 'Product Area',
    equipment: 'Product Tank',
    activationTime: new Date(Date.now() - 600000),
    returnedToNormalAt: new Date(Date.now() - 60000)
  }
];

// ISA-18.2 Compliant Alarm Summary Display
export default function ISA18AlarmDisplay() {
  const [alarms, setAlarms] = useState(generateMockAlarms());
  const [selectedAlarm, setSelectedAlarm] = useState(null);
  const [showShelved, setShowShelved] = useState(false);
  const [filter, setFilter] = useState('all');
  const [sortBy, setSortBy] = useState('priority'); // priority, time, area
  
  // Simulate alarm updates
  useEffect(() => {
    const interval = setInterval(() => {
      setAlarms(prev => prev.map(alarm => {
        // Simulate blinking for unacknowledged alarms
        if (alarm.state === ALARM_STATES.UNACKNOWLEDGED) {
          return { ...alarm, blink: !alarm.blink };
        }
        return alarm;
      }));
    }, 500);
    
    return () => clearInterval(interval);
  }, []);
  
  // Calculate alarm statistics
  const statistics = useMemo(() => {
    const stats = {
      total: alarms.length,
      byPriority: { critical: 0, high: 0, medium: 0, low: 0 },
      byState: {},
      unacknowledged: 0
    };
    
    alarms.forEach(alarm => {
      stats.byPriority[alarm.priority]++;
      stats.byState[alarm.state] = (stats.byState[alarm.state] || 0) + 1;
      if (alarm.state === ALARM_STATES.UNACKNOWLEDGED || 
          alarm.state === ALARM_STATES.RTN_UNACK) {
        stats.unacknowledged++;
      }
    });
    
    return stats;
  }, [alarms]);
  
  // Filter and sort alarms
  const displayedAlarms = useMemo(() => {
    let filtered = alarms;
    
    // Apply filters
    if (filter !== 'all') {
      filtered = filtered.filter(alarm => {
        if (filter === 'active') {
          return alarm.state === ALARM_STATES.UNACKNOWLEDGED || 
                 alarm.state === ALARM_STATES.ACKNOWLEDGED;
        }
        if (filter === 'unack') {
          return alarm.state === ALARM_STATES.UNACKNOWLEDGED || 
                 alarm.state === ALARM_STATES.RTN_UNACK;
        }
        return alarm.priority === filter;
      });
    }
    
    if (!showShelved) {
      filtered = filtered.filter(a => a.state !== ALARM_STATES.SHELVED);
    }
    
    // Sort alarms
    return filtered.sort((a, b) => {
      if (sortBy === 'priority') {
        const priorityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
        return priorityOrder[a.priority] - priorityOrder[b.priority];
      }
      if (sortBy === 'time') {
        return b.activationTime - a.activationTime;
      }
      if (sortBy === 'area') {
        return a.area.localeCompare(b.area);
      }
      return 0;
    });
  }, [alarms, filter, showShelved, sortBy]);
  
  const acknowledgeAlarm = (alarmName) => {
    setAlarms(prev => prev.map(alarm => {
      if (alarm.name === alarmName) {
        if (alarm.state === ALARM_STATES.UNACKNOWLEDGED) {
          return {
            ...alarm,
            state: ALARM_STATES.ACKNOWLEDGED,
            acknowledgedBy: 'Current User',
            acknowledgedAt: new Date()
          };
        }
        if (alarm.state === ALARM_STATES.RTN_UNACK) {
          return {
            ...alarm,
            state: ALARM_STATES.NORMAL
          };
        }
      }
      return alarm;
    }));
  };
  
  const getAlarmStyle = (alarm) => {
    const colors = ALARM_COLORS[alarm.priority];
    const shouldBlink = (alarm.state === ALARM_STATES.UNACKNOWLEDGED || 
                        alarm.state === ALARM_STATES.RTN_UNACK) && 
                        alarm.blink;
    
    return {
      backgroundColor: shouldBlink ? colors.background : 'transparent',
      color: shouldBlink ? colors.text : colors.background,
      border: `2px solid ${colors.background}`,
      transition: 'all 0.2s'
    };
  };
  
  const formatDuration = (date) => {
    const diff = Date.now() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    }
    return `${minutes}m`;
  };
  
  return (
    <div className="w-full max-w-7xl mx-auto p-4 space-y-4">
      {/* Alarm Summary Bar */}
      <Card className="bg-gray-800 text-white">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-6">
              <div className="flex items-center gap-2">
                <Bell className="h-5 w-5" />
                <span className="font-bold">ALARM SUMMARY</span>
              </div>
              
              <div className="flex items-center gap-4">
                {statistics.byPriority.critical > 0 && (
                  <Badge 
                    style={{ backgroundColor: ALARM_COLORS.critical.background }}
                    className="text-white px-3 py-1"
                  >
                    Critical: {statistics.byPriority.critical}
                  </Badge>
                )}
                {statistics.byPriority.high > 0 && (
                  <Badge 
                    style={{ backgroundColor: ALARM_COLORS.high.background }}
                    className="text-white px-3 py-1"
                  >
                    High: {statistics.byPriority.high}
                  </Badge>
                )}
                {statistics.unacknowledged > 0 && (
                  <Badge 
                    variant="outline"
                    className="border-white text-white px-3 py-1 animate-pulse"
                  >
                    Unack: {statistics.unacknowledged}
                  </Badge>
                )}
              </div>
            </div>
            
            <div className="text-sm">
              Total Active: {statistics.total} | {new Date().toLocaleTimeString()}
            </div>
          </div>
        </CardContent>
      </Card>
      
      {/* Filter Controls */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">Filter:</span>
              <div className="flex gap-2">
                {['all', 'active', 'unack', 'critical', 'high', 'medium', 'low'].map(f => (
                  <button
                    key={f}
                    onClick={() => setFilter(f)}
                    className={`px-3 py-1 rounded text-sm ${
                      filter === f 
                        ? 'bg-blue-500 text-white' 
                        : 'bg-gray-200 hover:bg-gray-300'
                    }`}
                  >
                    {f.charAt(0).toUpperCase() + f.slice(1)}
                  </button>
                ))}
              </div>
            </div>
            
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">Sort:</span>
                <select 
                  value={sortBy} 
                  onChange={(e) => setSortBy(e.target.value)}
                  className="px-2 py-1 border rounded text-sm"
                >
                  <option value="priority">Priority</option>
                  <option value="time">Time</option>
                  <option value="area">Area</option>
                </select>
              </div>
              
              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={showShelved}
                  onChange={(e) => setShowShelved(e.target.checked)}
                />
                Show Shelved
              </label>
            </div>
          </div>
        </CardContent>
      </Card>
      
      {/* Alarm List */}
      <Card>
        <CardHeader>
          <CardTitle>Active Alarms</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-100 border-b">
                <tr>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">Priority</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">Time</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">Tag</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">Description</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">Value</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">Area</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">State</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase">Actions</th>
                </tr>
              </thead>
              <tbody>
                {displayedAlarms.map((alarm) => {
                  const Icon = ALARM_COLORS[alarm.priority].icon;
                  
                  return (
                    <tr 
                      key={alarm.name}
                      style={getAlarmStyle(alarm)}
                      className="border-b hover:bg-gray-50 cursor-pointer"
                      onClick={() => setSelectedAlarm(alarm)}
                    >
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <Icon className="h-4 w-4" />
                          <span className="text-sm font-medium uppercase">
                            {alarm.priority}
                          </span>
                        </div>
                      </td>
                      <td className="px-4 py-3 text-sm">
                        {formatDuration(alarm.activationTime)}
                      </td>
                      <td className="px-4 py-3 text-sm font-mono">{alarm.tagName}</td>
                      <td className="px-4 py-3 text-sm">{alarm.description}</td>
                      <td className="px-4 py-3 text-sm font-mono">
                        {alarm.value} {alarm.units}
                      </td>
                      <td className="px-4 py-3 text-sm">{alarm.area}</td>
                      <td className="px-4 py-3 text-sm">{alarm.state}</td>
                      <td className="px-4 py-3">
                        {(alarm.state === ALARM_STATES.UNACKNOWLEDGED || 
                          alarm.state === ALARM_STATES.RTN_UNACK) && (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              acknowledgeAlarm(alarm.name);
                            }}
                            className="px-3 py-1 bg-blue-500 text-white rounded text-xs hover:bg-blue-600"
                          >
                            ACK
                          </button>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
      
      {/* Alarm Detail Modal */}
      {selectedAlarm && (
        <div 
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          onClick={() => setSelectedAlarm(null)}
        >
          <Card 
            className="w-full max-w-2xl mx-4"
            onClick={(e) => e.stopPropagation()}
          >
            <CardHeader 
              style={{ backgroundColor: ALARM_COLORS[selectedAlarm.priority].background }}
              className="text-white"
            >
              <CardTitle className="flex items-center justify-between">
                <span>Alarm Detail: {selectedAlarm.tagName}</span>
                <button
                  onClick={() => setSelectedAlarm(null)}
                  className="text-white hover:text-gray-200"
                >
                  ✕
                </button>
              </CardTitle>
            </CardHeader>
            <CardContent className="p-6 space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-gray-600">Alarm Name</p>
                  <p className="font-medium">{selectedAlarm.name}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Priority</p>
                  <p className="font-medium uppercase">{selectedAlarm.priority}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Description</p>
                  <p className="font-medium">{selectedAlarm.description}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Current State</p>
                  <p className="font-medium">{selectedAlarm.state}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Current Value</p>
                  <p className="font-medium font-mono">
                    {selectedAlarm.value} {selectedAlarm.units}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Setpoint</p>
                  <p className="font-medium font-mono">
                    {selectedAlarm.setpoint} {selectedAlarm.units}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Area / Equipment</p>
                  <p className="font-medium">
                    {selectedAlarm.area} / {selectedAlarm.equipment}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Activation Time</p>
                  <p className="font-medium">
                    {selectedAlarm.activationTime.toLocaleString()}
                  </p>
                </div>
              </div>
              
              <div className="border-t pt-4">
                <p className="text-sm text-gray-600 mb-2">Consequence</p>
                <p className="text-red-600 font-medium">{selectedAlarm.consequence}</p>
              </div>
              
              <div className="border-t pt-4">
                <p className="text-sm text-gray-600 mb-2">Corrective Action</p>
                <p className="font-medium whitespace-pre-line">{selectedAlarm.correctiveAction}</p>
              </div>
              
              {selectedAlarm.acknowledgedBy && (
                <div className="border-t pt-4">
                  <p className="text-sm text-gray-600">Acknowledged By</p>
                  <p className="font-medium">
                    {selectedAlarm.acknowledgedBy} at {selectedAlarm.acknowledgedAt?.toLocaleString()}
                  </p>
                </div>
              )}
              
              <div className="flex justify-end gap-2 pt-4">
                {(selectedAlarm.state === ALARM_STATES.UNACKNOWLEDGED || 
                  selectedAlarm.state === ALARM_STATES.RTN_UNACK) && (
                  <button
                    onClick={() => {
                      acknowledgeAlarm(selectedAlarm.name);
                      setSelectedAlarm(null);
                    }}
                    className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                  >
                    Acknowledge
                  </button>
                )}
                <button
                  onClick={() => setSelectedAlarm(null)}
                  className="px-4 py-2 bg-gray-300 text-gray-700 rounded hover:bg-gray-400"
                >
                  Close
                </button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}

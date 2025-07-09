import React, { useState, useEffect } from 'react';
import ISA101Trend from '../ISA101Trend';
import HOAComponent from '../HOAComponent';
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import { AlertTriangle, Activity, Droplet, Gauge } from 'lucide-react';

interface Signal {
  name: string;
  value: any;
  type: string;
  quality?: string;
  timestamp?: number;
}

interface HMIDashboardProps {
  signals: Map<string, any>;
  connected: boolean;
  isISA101Mode?: boolean;
}

export default function HMIDashboard({ signals, connected, isISA101Mode = true }: HMIDashboardProps) {
  const [trendData, setTrendData] = useState<any>({
    tankLevel: [],
    flowRate: [],
    pressure: []
  });

  // Update trend data when signals change
  useEffect(() => {
    const tankLevel = signals.get('tank.level_feet');
    const flowRate = signals.get('well.flow_rate');
    const pressure = signals.get('system.pressure');
    
    if (tankLevel !== undefined || flowRate !== undefined || pressure !== undefined) {
      const now = Date.now();
      
      setTrendData((prev: any) => ({
        tankLevel: tankLevel !== undefined ? [...prev.tankLevel.slice(-300), {
          timestamp: now,
          value: tankLevel,
          quality: 'good'
        }] : prev.tankLevel,
        flowRate: flowRate !== undefined ? [...prev.flowRate.slice(-300), {
          timestamp: now,
          value: flowRate,
          quality: 'good'
        }] : prev.flowRate,
        pressure: pressure !== undefined ? [...prev.pressure.slice(-300), {
          timestamp: now,
          value: pressure,
          quality: 'good'
        }] : prev.pressure
      }));
    }
  }, [signals]);

  const getSignalValue = (signalName: string, defaultValue: any = '--') => {
    return signals.get(signalName) ?? defaultValue;
  };

  const getAlarmClass = (value: number, low: number, high: number, critical?: number) => {
    if (critical !== undefined && value >= critical) return 'isa101-alarm-critical';
    if (value >= high) return 'isa101-alarm-high';
    if (value <= low) return 'isa101-alarm-low';
    return '';
  };

  return (
    <div className="h-full overflow-auto p-4">
      <div className="grid grid-cols-12 gap-4">
        {/* System Status */}
        <div className="col-span-12 md:col-span-3">
          <Card className="isa101-container">
            <CardHeader className="isa101-panel-header">
              <CardTitle className="text-sm flex items-center gap-2">
                <Activity className="h-4 w-4" />
                System Status
              </CardTitle>
            </CardHeader>
            <CardContent className="p-4 space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm">Connection:</span>
                <Badge className={connected ? 'isa101-status-running' : 'isa101-status-stopped'}>
                  {connected ? 'ONLINE' : 'OFFLINE'}
                </Badge>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Scan Time:</span>
                <span className="isa101-value">100 ms</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Active Alarms:</span>
                <span className={`isa101-value ${getSignalValue('system.alarm_count', 0) > 0 ? 'isa101-alarm-high' : ''}`}>
                  {getSignalValue('system.alarm_count', 0)}
                </span>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Tank Status */}
        <div className="col-span-12 md:col-span-3">
          <Card className="isa101-container">
            <CardHeader className="isa101-panel-header">
              <CardTitle className="text-sm flex items-center gap-2">
                <Droplet className="h-4 w-4" />
                Tank T-101
              </CardTitle>
            </CardHeader>
            <CardContent className="p-4 space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm">Level:</span>
                <span className={`isa101-value ${getAlarmClass(getSignalValue('tank.level_feet', 0), 5, 20, 23)}`}>
                  {getSignalValue('tank.level_feet', '--').toFixed(1)} ft
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Percent:</span>
                <span className="isa101-value">
                  {getSignalValue('tank.level_percent', '--').toFixed(0)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Temperature:</span>
                <span className="isa101-value">
                  {getSignalValue('tank.temperature', '--').toFixed(1)}°F
                </span>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Pump Control */}
        <div className="col-span-12 md:col-span-3">
          <Card className="isa101-container">
            <CardHeader className="isa101-panel-header">
              <CardTitle className="text-sm flex items-center gap-2">
                <Gauge className="h-4 w-4" />
                Well Pump P-101
              </CardTitle>
            </CardHeader>
            <CardContent className="p-4 space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm">Status:</span>
                <Badge className={getSignalValue('well.running') ? 'isa101-status-running' : 'isa101-status-stopped'}>
                  {getSignalValue('well.running') ? 'RUNNING' : 'STOPPED'}
                </Badge>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Flow Rate:</span>
                <span className="isa101-value">
                  {getSignalValue('well.flow_rate', 0).toFixed(0)} gpm
                </span>
              </div>
              <div className="mt-2">
                <HOAComponent
                  id="pump_p101_hoa"
                  value={getSignalValue('well.hoa_mode', 'off')}
                  size="small"
                  onModeChange={(mode, id) => {
                    console.log(`Setting ${id} to ${mode} mode`);
                    // Send command to PETRA
                  }}
                />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Alarms */}
        <div className="col-span-12 md:col-span-3">
          <Card className="isa101-container">
            <CardHeader className="isa101-panel-header">
              <CardTitle className="text-sm flex items-center gap-2">
                <AlertTriangle className="h-4 w-4" />
                Active Alarms
              </CardTitle>
            </CardHeader>
            <CardContent className="p-4">
              <div className="space-y-2 max-h-32 overflow-auto">
                {getSignalValue('tank.level_feet', 0) > 20 && (
                  <div className="isa101-alarm-high p-2 text-xs">
                    Tank Level High: {getSignalValue('tank.level_feet', 0).toFixed(1)} ft
                  </div>
                )}
                {getSignalValue('tank.level_feet', 0) < 5 && (
                  <div className="isa101-alarm-low p-2 text-xs">
                    Tank Level Low: {getSignalValue('tank.level_feet', 0).toFixed(1)} ft
                  </div>
                )}
                {!connected && (
                  <div className="isa101-alarm-critical p-2 text-xs">
                    System: Connection Lost
                  </div>
                )}
                {connected && getSignalValue('tank.level_feet', 0) >= 5 && getSignalValue('tank.level_feet', 0) <= 20 && (
                  <div className="text-xs text-gray-600 p-2">
                    No active alarms
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Main Trend */}
        <div className="col-span-12 md:col-span-8">
          <Card className="isa101-container">
            <ISA101Trend
              series={[
                {
                  name: 'Tank Level',
                  color: '#000080',
                  unit: 'ft',
                  data: trendData.tankLevel,
                  min: 0,
                  max: 25
                },
                {
                  name: 'Flow Rate',
                  color: '#008000',
                  unit: 'gpm',
                  data: trendData.flowRate,
                  min: 0,
                  max: 3000
                }
              ]}
              title="Process Overview"
              height={400}
              timeRange={300}
            />
          </Card>
        </div>

        {/* Process Values */}
        <div className="col-span-12 md:col-span-4">
          <Card className="isa101-container h-full">
            <CardHeader className="isa101-panel-header">
              <CardTitle className="text-sm">Process Values</CardTitle>
            </CardHeader>
            <CardContent className="p-4">
              <div className="space-y-3">
                <div className="isa101-panel">
                  <h4 className="text-xs font-semibold mb-2">Tank System</h4>
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>Level SP:</div>
                    <div className="isa101-value">{getSignalValue('tank.level_setpoint', '--')} ft</div>
                    <div>High Limit:</div>
                    <div className="isa101-value">20.0 ft</div>
                    <div>Low Limit:</div>
                    <div className="isa101-value">5.0 ft</div>
                  </div>
                </div>
                
                <div className="isa101-panel">
                  <h4 className="text-xs font-semibold mb-2">Pump System</h4>
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>Start Level:</div>
                    <div className="isa101-value">{getSignalValue('well.start_level', '--')} ft</div>
                    <div>Stop Level:</div>
                    <div className="isa101-value">{getSignalValue('well.stop_level', '--')} ft</div>
                    <div>Run Time:</div>
                    <div className="isa101-value">{getSignalValue('well.runtime_hours', '--')} hrs</div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

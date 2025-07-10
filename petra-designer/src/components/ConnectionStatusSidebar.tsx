import React, { useState, useEffect } from 'react';
import { Activity, WifiOff, Wifi, Server, Clock, AlertTriangle } from 'lucide-react';

interface ConnectionInfo {
  status: 'connected' | 'disconnected' | 'connecting' | 'error';
  latency: number;
  uptime: number;
  lastError?: string;
  messageRate: number;
  reconnectAttempts: number;
}

export default function ConnectionStatusSidebar({ 
  connectionInfo,
  isOpen,
  onToggle 
}: {
  connectionInfo: ConnectionInfo;
  isOpen: boolean;
  onToggle: () => void;
}) {
  const [uptimeString, setUptimeString] = useState('00:00:00');

  useEffect(() => {
    const formatUptime = (seconds: number) => {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      const secs = seconds % 60;
      return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    };

    const interval = setInterval(() => {
      setUptimeString(formatUptime(connectionInfo.uptime));
    }, 1000);

    return () => clearInterval(interval);
  }, [connectionInfo.uptime]);

  const getStatusColor = () => {
    switch (connectionInfo.status) {
      case 'connected':
        return '#00C800';
      case 'connecting':
        return '#FFD700';
      case 'error':
        return '#FF0000';
      default:
        return '#808080';
    }
  };

  const getStatusIcon = () => {
    switch (connectionInfo.status) {
      case 'connected':
        return <Wifi size={16} />;
      case 'connecting':
        return <Activity size={16} className="animate-pulse" />;
      case 'error':
        return <AlertTriangle size={16} />;
      default:
        return <WifiOff size={16} />;
    }
  };

  return (
    <>
      {/* Compact Status Bar */}
      <div 
        className="isa101-connection-status cursor-pointer"
        onClick={onToggle}
        style={{
          position: 'fixed',
          bottom: 0,
          right: 0,
          zIndex: 1000,
          borderLeft: `1px solid var(--isa101-border)`,
          borderTop: `1px solid var(--isa101-border)`
        }}
      >
        <div className="flex items-center gap-2">
          <div 
            className="isa101-connection-indicator"
            style={{ backgroundColor: getStatusColor() }}
          />
          {getStatusIcon()}
          <span className="font-medium">
            PETRA {connectionInfo.status.toUpperCase()}
          </span>
          {connectionInfo.status === 'connected' && (
            <span className="text-xs opacity-75">
              {connectionInfo.latency}ms
            </span>
          )}
        </div>
      </div>

      {/* Detailed Sidebar */}
      <div
        className={`isa101-sidebar fixed right-0 top-0 h-full transition-transform duration-200 ${
          isOpen ? 'translate-x-0' : 'translate-x-full'
        }`}
        style={{
          width: '300px',
          zIndex: 999,
          boxShadow: isOpen ? '-3px 0 5px rgba(0,0,0,0.1)' : 'none'
        }}
      >
        <div className="isa101-panel-header flex justify-between items-center">
          <h3 className="font-semibold">Connection Details</h3>
          <button
            onClick={onToggle}
            className="isa101-button text-xs px-2 py-1"
          >
            ✕
          </button>
        </div>

        <div className="p-4 space-y-4">
          {/* Status Section */}
          <div className="isa101-panel">
            <h4 className="font-medium mb-2 flex items-center gap-2">
              <Server size={14} />
              Server Status
            </h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Status:</span>
                <span className="font-medium flex items-center gap-1">
                  {getStatusIcon()}
                  {connectionInfo.status.toUpperCase()}
                </span>
              </div>
              <div className="flex justify-between">
                <span>Endpoint:</span>
                <span className="font-mono text-xs">ws://localhost:8080</span>
              </div>
              <div className="flex justify-between">
                <span>Uptime:</span>
                <span className="font-mono">{uptimeString}</span>
              </div>
            </div>
          </div>

          {/* Performance Section */}
          <div className="isa101-panel">
            <h4 className="font-medium mb-2 flex items-center gap-2">
              <Activity size={14} />
              Performance
            </h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Latency:</span>
                <span className={`font-mono ${connectionInfo.latency > 100 ? 'text-red-600' : ''}`}>
                  {connectionInfo.latency}ms
                </span>
              </div>
              <div className="flex justify-between">
                <span>Message Rate:</span>
                <span className="font-mono">{connectionInfo.messageRate}/s</span>
              </div>
              <div className="flex justify-between">
                <span>Reconnect Attempts:</span>
                <span className="font-mono">{connectionInfo.reconnectAttempts}</span>
              </div>
            </div>
          </div>

          {/* Error Section */}
          {connectionInfo.lastError && (
            <div className="isa101-panel" style={{ borderColor: 'var(--isa101-alarm-high)' }}>
              <h4 className="font-medium mb-2 flex items-center gap-2 text-orange-600">
                <AlertTriangle size={14} />
                Last Error
              </h4>
              <p className="text-sm text-red-600">{connectionInfo.lastError}</p>
            </div>
          )}

          {/* Connection Actions */}
          <div className="space-y-2">
            <button
              className="isa101-button w-full"
              onClick={() => console.log('Reconnecting...')}
              disabled={connectionInfo.status === 'connected'}
            >
              Reconnect
            </button>
            <button
              className="isa101-button w-full"
              onClick={() => console.log('Opening diagnostics...')}
            >
              Diagnostics
            </button>
          </div>
        </div>
      </div>
    </>
  );
}

// Example usage with state management
export function ConnectionStatusExample() {
  const [isOpen, setIsOpen] = useState(false);
  const [connectionInfo, setConnectionInfo] = useState<ConnectionInfo>({
    status: 'connected',
    latency: 23,
    uptime: 3661,
    messageRate: 15,
    reconnectAttempts: 0
  });

  // Simulate connection changes
  useEffect(() => {
    const interval = setInterval(() => {
      setConnectionInfo(prev => ({
        ...prev,
        latency: Math.floor(Math.random() * 50) + 10,
        messageRate: Math.floor(Math.random() * 20) + 5,
        uptime: prev.uptime + 1
      }));
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <ConnectionStatusSidebar
      connectionInfo={connectionInfo}
      isOpen={isOpen}
      onToggle={() => setIsOpen(!isOpen)}
    />
  );
}

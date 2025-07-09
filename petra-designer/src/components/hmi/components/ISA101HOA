import React, { useState } from 'react';

export interface HOAComponentProps {
  id: string;
  value?: 'hand' | 'off' | 'auto';
  onChange?: (value: 'hand' | 'off' | 'auto') => void;
  disabled?: boolean;
  size?: 'small' | 'medium' | 'large';
  showLabels?: boolean;
  onModeChange?: (mode: 'hand' | 'off' | 'auto', signalId: string) => void;
}

export default function HOAComponent({
  id,
  value = 'off',
  onChange,
  disabled = false,
  size = 'medium',
  showLabels = true,
  onModeChange
}: HOAComponentProps) {
  const [currentMode, setCurrentMode] = useState<'hand' | 'off' | 'auto'>(value);

  const handleModeChange = (newMode: 'hand' | 'off' | 'auto') => {
    if (disabled) return;
    
    setCurrentMode(newMode);
    onChange?.(newMode);
    onModeChange?.(newMode, id);
  };

  const getSizeClasses = () => {
    switch (size) {
      case 'small':
        return 'text-xs py-1 px-2';
      case 'large':
        return 'text-lg py-3 px-6';
      default:
        return 'text-sm py-2 px-4';
    }
  };

  const getLabel = (mode: string) => {
    if (!showLabels) {
      return mode.charAt(0).toUpperCase();
    }
    switch (mode) {
      case 'hand':
        return 'HAND';
      case 'off':
        return 'OFF';
      case 'auto':
        return 'AUTO';
      default:
        return mode.toUpperCase();
    }
  };

  return (
    <div className="isa101-hoa inline-flex rounded-none">
      <button
        className={`isa101-hoa-button hand ${currentMode === 'hand' ? 'active' : ''} ${getSizeClasses()}`}
        onClick={() => handleModeChange('hand')}
        disabled={disabled}
        title="Manual (Hand) Mode - Local control only"
      >
        {getLabel('hand')}
      </button>
      <button
        className={`isa101-hoa-button off ${currentMode === 'off' ? 'active' : ''} ${getSizeClasses()}`}
        onClick={() => handleModeChange('off')}
        disabled={disabled}
        title="Off Mode - Equipment stopped"
      >
        {getLabel('off')}
      </button>
      <button
        className={`isa101-hoa-button auto ${currentMode === 'auto' ? 'active' : ''} ${getSizeClasses()}`}
        onClick={() => handleModeChange('auto')}
        disabled={disabled}
        title="Automatic Mode - PLC control"
      >
        {getLabel('auto')}
      </button>
    </div>
  );
}

// Example usage component
export function HOAExample() {
  const [mode, setMode] = useState<'hand' | 'off' | 'auto'>('off');

  return (
    <div className="p-4 space-y-4">
      <h3 className="text-lg font-semibold">H-O-A Control Examples</h3>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-2">Pump P-101 Control</label>
          <HOAComponent
            id="pump_p101_hoa"
            value={mode}
            onChange={setMode}
            onModeChange={(newMode, signalId) => {
              console.log(`${signalId} changed to ${newMode} mode`);
            }}
          />
          <p className="mt-2 text-sm text-gray-600">Current mode: {mode}</p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Small HOA (Compact View)</label>
          <HOAComponent
            id="valve_v201_hoa"
            size="small"
            value="auto"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Large HOA (Operator Panel)</label>
          <HOAComponent
            id="motor_m301_hoa"
            size="large"
            value="hand"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Disabled HOA (Maintenance Mode)</label>
          <HOAComponent
            id="pump_p401_hoa"
            value="off"
            disabled={true}
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Icon Only HOA</label>
          <HOAComponent
            id="fan_f501_hoa"
            showLabels={false}
            value="auto"
          />
        </div>
      </div>
    </div>
  );
}

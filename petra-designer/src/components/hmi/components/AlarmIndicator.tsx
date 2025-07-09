// Individual alarm indicator
import React from 'react';
import { Alarm } from '../types/alarm.types';

interface AlarmIndicatorProps {
  alarm: Alarm;
}

export default function AlarmIndicator({ alarm }: AlarmIndicatorProps) {
  return (
    <div className="alarm-indicator">
      {alarm.name} - {alarm.state}
    </div>
  );
}

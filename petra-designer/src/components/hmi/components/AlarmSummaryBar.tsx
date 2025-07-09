// Top banner showing alarm counts by priority
import React from 'react';
import { Alarm } from '../types/alarm.types';

interface AlarmSummaryProps {
  statistics: {
    critical: number;
    high: number;
    medium: number;
    low: number;
    unacknowledged: number;
  };
}

export default function AlarmSummaryBar({ statistics }: AlarmSummaryProps) {
  return (
    <div className="alarm-summary-bar">
      {/* Displays critical count, high count, unack count */}
      <span>Critical: {statistics.critical}</span>
      <span>High: {statistics.high}</span>
      <span>Unack: {statistics.unacknowledged}</span>
    </div>
  );
}

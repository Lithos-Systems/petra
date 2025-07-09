// Main alarm display component that combines all sub-components
import React from 'react';
import AlarmSummaryBar from './AlarmSummaryBar';
import AlarmFilters from './AlarmFilters';
import AlarmList from './AlarmList';
import AlarmDetailModal from './AlarmDetailModal';
import { useAlarms } from '../hooks/useAlarms';

export default function ISA18AlarmDisplay() {
  const { alarms, acknowledgeAlarm } = useAlarms();
  const statistics = {
    critical: alarms.filter(a => a.priority === 'critical').length,
    high: alarms.filter(a => a.priority === 'high').length,
    medium: alarms.filter(a => a.priority === 'medium').length,
    low: alarms.filter(a => a.priority === 'low').length,
    unacknowledged: alarms.filter(a => a.state === 'Unacknowledged').length
  };

  return (
    <div>
      <AlarmSummaryBar statistics={statistics} />
      <AlarmFilters filter="all" setFilter={() => {}} />
      <AlarmList alarms={alarms} onAlarmSelect={() => {}} />
      <AlarmDetailModal alarm={null} onClose={() => {}} />
    </div>
  );
}

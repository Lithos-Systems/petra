// Table/list of active alarms with ISA-18.2 compliant display
import React from 'react';
import { Alarm } from '../types/alarm.types';

interface AlarmListProps {
  alarms: Alarm[];
  onAlarmSelect?: (alarm: Alarm) => void;
}

export default function AlarmList({ alarms, onAlarmSelect }: AlarmListProps) {
  return (
    <table className="alarm-list">
      <tbody>
        {alarms.map(alarm => (
          <tr key={alarm.name} onClick={() => onAlarmSelect?.(alarm)}>
            <td>{alarm.priority}</td>
            <td>{alarm.tagName}</td>
            <td>{alarm.description}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

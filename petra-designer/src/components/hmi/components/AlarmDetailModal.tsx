// Alarm detail popup
import React from 'react';
import { Alarm } from '../types/alarm.types';

interface AlarmDetailModalProps {
  alarm: Alarm | null;
  onClose: () => void;
}

export default function AlarmDetailModal({ alarm, onClose }: AlarmDetailModalProps) {
  if (!alarm) return null;
  return (
    <div className="alarm-detail-modal">
      <div className="modal-content">
        <button onClick={onClose}>Close</button>
        <pre>{JSON.stringify(alarm, null, 2)}</pre>
      </div>
    </div>
  );
}

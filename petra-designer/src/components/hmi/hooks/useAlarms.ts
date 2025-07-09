// Hook for alarm data management
import { useState, useEffect } from 'react';
import { Alarm } from '../types/alarm.types';

export function useAlarms() {
  const [alarms, setAlarms] = useState<Alarm[]>([]);

  // Subscribe to alarm updates from signal bus
  useEffect(() => {
    const subscription = subscribeToAlarms((update: Alarm[]) => {
      setAlarms(update);
    });

    return () => subscription.unsubscribe();
  }, []);

  const acknowledgeAlarm = async (alarmName: string) => {
    // Send acknowledgment to backend
  };

  return { alarms, acknowledgeAlarm };
}

// Placeholder subscribe function
function subscribeToAlarms(cb: (a: Alarm[]) => void) {
  const interval = setInterval(() => cb([]), 1000);
  return { unsubscribe: () => clearInterval(interval) };
}

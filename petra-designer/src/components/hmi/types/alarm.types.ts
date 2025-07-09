// TypeScript interfaces matching Rust structures
export interface Alarm {
  name: string;
  description: string;
  tagName: string;
  priority: AlarmPriority;
  state: AlarmState;
  value: number;
  units: string;
  area: string;
  equipment: string;
  consequence: string;
  correctiveAction: string;
  activationTime: Date;
  acknowledgedBy?: string;
  acknowledgedAt?: Date;
}

export enum AlarmPriority {
  Critical = 'critical',
  High = 'high',
  Medium = 'medium',
  Low = 'low'
}

export enum AlarmState {
  Normal = 'Normal',
  Unacknowledged = 'Unacknowledged',
  Acknowledged = 'Acknowledged',
  RTNUnacknowledged = 'RTN-Unack',
  Suppressed = 'Suppressed',
  OutOfService = 'Out of Service',
  Shelved = 'Shelved'
}

// Sorting/filtering logic for alarms
import { Alarm } from '../types/alarm.types';

export function sortByPriority(a: Alarm, b: Alarm): number {
  const order = { critical: 0, high: 1, medium: 2, low: 3 } as any;
  return order[a.priority] - order[b.priority];
}

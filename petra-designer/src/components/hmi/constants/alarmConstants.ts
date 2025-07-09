// ISA-18.2 compliant constants
export const ALARM_COLORS = {
  critical: {
    background: '#FF0000',
    text: '#FFFFFF',
    blink: true
  },
  high: {
    background: '#FF8C00',
    text: '#FFFFFF',
    blink: true
  },
  medium: {
    background: '#FFFF00',
    text: '#000000',
    blink: true
  },
  low: {
    background: '#00FFFF',
    text: '#000000',
    blink: false
  }
};

export const ALARM_BLINK_RATE = 1000; // 1Hz per ISA-18.2
export const ALARM_FLOOD_THRESHOLD = 10; // per 10 minutes

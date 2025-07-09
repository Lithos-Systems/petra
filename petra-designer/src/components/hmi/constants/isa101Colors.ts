// petra-designer/src/components/hmi/constants/isa101Colors.ts
// ISA-101 Standard Colors for HMI Graphics

const ISA101Colors = {
  // Equipment and Process Lines
  equipmentOutline: '#000000',      // Black - equipment outlines
  equipmentFill: '#E6E6E6',         // Light gray - equipment fill
  processLine: '#000000',           // Black - process lines
  
  // Process States
  running: '#00FF00',               // Green - running/active/open
  stopped: '#808080',               // Gray - stopped/inactive/closed
  transitioning: '#FFFF00',         // Yellow - transitioning state
  
  // Alarm Priority Colors (per ISA-18.2)
  alarmCritical: '#FF0000',         // Red - Critical/Safety alarms (Priority 1)
  alarmHigh: '#FF8C00',             // Orange - High priority alarms (Priority 2)
  alarmMedium: '#FFFF00',           // Yellow - Medium priority alarms (Priority 3)
  alarmLow: '#00FFFF',              // Cyan - Low priority alarms (Priority 4)
  alarmMessage: '#C0C0C0',          // Gray - Messages/Events
  
  // Special States
  fault: '#FF0000',                 // Red - fault condition
  interlocked: '#FF8C00',           // Orange - interlocks
  manual: '#9370DB',                // Purple - manual mode
  
  // Process Values and Text
  processValue: '#000000',          // Black - primary text and values
  setpoint: '#0000FF',              // Blue - setpoints
  textSecondary: '#404040',         // Dark gray - secondary text
  textAlarm: '#FFFFFF',             // White - text on alarm backgrounds
  
  // Liquid/Material Colors (subdued per ISA-101)
  liquidNormal: '#87CEEB',          // Sky blue - normal liquid
  liquidHot: '#FFB6C1',             // Light red - hot liquid
  liquidChemical: '#DDA0DD',        // Plum - chemical
  liquidOil: '#8B7355',             // Sienna - oil products
  
  // Background Colors
  background: '#F0F0F0',            // Light gray - main background
  containerBackground: '#FFFFFF',    // White - container backgrounds
  inputBackground: '#FFFFFF',        // White - input fields
  
  // UI Elements
  toolbarBg: '#E6E6E6',             // Light gray - toolbar background
  toolbarBorder: '#C0C0C0',         // Silver - toolbar borders
  buttonBg: '#FFFFFF',              // White - button background
  buttonHover: '#D6D6D6',           // Light gray - button hover
  buttonActive: '#B0B0B0',          // Gray - button active/pressed
  
  // Grid and Guidelines
  gridLine: '#D0D0D0',              // Light gray - grid lines
  guideline: '#0080FF',             // Blue - alignment guides
  selection: '#0080FF',             // Blue - selection highlight
  
  // Trend Colors (for multi-pen trends)
  trend1: '#0000FF',                // Blue - trend pen 1
  trend2: '#FF0000',                // Red - trend pen 2
  trend3: '#00FF00',                // Green - trend pen 3
  trend4: '#FF00FF',                // Magenta - trend pen 4
  trend5: '#00FFFF',                // Cyan - trend pen 5
  trend6: '#FFFF00',                // Yellow - trend pen 6
  trend7: '#FF8000',                // Orange - trend pen 7
  trend8: '#8000FF',                // Purple - trend pen 8
  
  // Gauge and Scale Colors
  scale: '#666666',                 // Dark gray - scale markings
  needleNormal: '#000000',          // Black - normal needle
  needleAlarm: '#FF0000',           // Red - alarm condition needle
  
  // Status Indicators
  statusGood: '#00FF00',            // Green - good/normal
  statusWarning: '#FFFF00',         // Yellow - warning
  statusBad: '#FF0000',             // Red - bad/critical
  statusUncertain: '#808080',       // Gray - uncertain/unknown
  
  // Focus and Interaction
  focus: '#0080FF',                 // Blue - keyboard focus
  hover: '#0080FF33',               // Blue with transparency - hover state
  active: '#0080FF66',              // Blue with transparency - active state
}

// Validate colors are properly formatted
Object.entries(ISA101Colors).forEach(([key, value]) => {
  if (!/^#[0-9A-F]{6}$|^#[0-9A-F]{8}$/i.test(value)) {
    console.warn(`ISA101Colors.${key} has invalid color format: ${value}`)
  }
})

export default ISA101Colors

// Export grouped color sets for convenience
export const ISA101AlarmColors = {
  critical: ISA101Colors.alarmCritical,
  high: ISA101Colors.alarmHigh,
  medium: ISA101Colors.alarmMedium,
  low: ISA101Colors.alarmLow,
  message: ISA101Colors.alarmMessage,
}

export const ISA101ProcessStates = {
  running: ISA101Colors.running,
  stopped: ISA101Colors.stopped,
  transitioning: ISA101Colors.transitioning,
  fault: ISA101Colors.fault,
  interlocked: ISA101Colors.interlocked,
  manual: ISA101Colors.manual,
}

export const ISA101LiquidColors = {
  water: ISA101Colors.liquidNormal,
  hot: ISA101Colors.liquidHot,
  chemical: ISA101Colors.liquidChemical,
  oil: ISA101Colors.liquidOil,
}

// Helper function to get alarm color by priority
export function getAlarmColorByPriority(priority: 1 | 2 | 3 | 4 | 'message'): string {
  switch (priority) {
    case 1: return ISA101Colors.alarmCritical
    case 2: return ISA101Colors.alarmHigh
    case 3: return ISA101Colors.alarmMedium
    case 4: return ISA101Colors.alarmLow
    case 'message': return ISA101Colors.alarmMessage
    default: return ISA101Colors.alarmMessage
  }
}

// Helper function to determine if color should have light or dark text
export function getContrastTextColor(backgroundColor: string): string {
  // Simple algorithm - could be enhanced with proper luminance calculation
  const rgb = parseInt(backgroundColor.slice(1), 16)
  const r = (rgb >> 16) & 0xff
  const g = (rgb >> 8) & 0xff
  const b = (rgb >> 0) & 0xff
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255
  
  return luminance > 0.5 ? ISA101Colors.processValue : ISA101Colors.textAlarm
}

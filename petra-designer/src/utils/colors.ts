// src/utils/colors.ts

/**
 * Get color for signal/value types
 */
export function getTypeColor(type: string): string {
  switch (type) {
    case 'bool':
      return '#22c55e' // green-500
    case 'int':
      return '#f59e0b' // amber-500
    case 'float':
      return '#3b82f6' // blue-500
    case 'string':
      return '#8b5cf6' // violet-500
    case 'any':
      return '#6b7280' // gray-500
    default:
      return '#6b7280' // gray-500
  }
}

/**
 * Get background color for different node types
 */
export function getNodeBgColor(type: string): string {
  switch (type) {
    case 'signal':
      return '#dbeafe' // blue-100
    case 'block':
      return '#d1fae5' // green-100
    case 'twilio':
      return '#f3e8ff' // purple-100
    case 'mqtt':
      return '#fed7aa' // orange-100
    case 's7':
      return '#fee2e2' // red-100
    case 'alarm':
      return '#fef3c7' // yellow-100
    case 'email':
      return '#e0e7ff' // indigo-100
    default:
      return '#f3f4f6' // gray-100
  }
}

/**
 * Get border color for different node types
 */
export function getNodeBorderColor(type: string, selected: boolean = false): string {
  if (selected) {
    return '#E95044' // petra-500
  }
  
  switch (type) {
    case 'signal':
      return '#60a5fa' // blue-400
    case 'block':
      return '#34d399' // green-400
    case 'twilio':
      return '#c084fc' // purple-400
    case 'mqtt':
      return '#fb923c' // orange-400
    case 's7':
      return '#f87171' // red-400
    case 'alarm':
      return '#fbbf24' // yellow-400
    case 'email':
      return '#818cf8' // indigo-400
    default:
      return '#9ca3af' // gray-400
  }
}

/**
 * Get text color for different severity levels
 */
export function getSeverityColor(severity: string): string {
  switch (severity.toLowerCase()) {
    case 'critical':
      return '#dc2626' // red-600
    case 'high':
      return '#ea580c' // orange-600
    case 'medium':
      return '#ca8a04' // yellow-600
    case 'low':
      return '#059669' // green-600
    case 'info':
      return '#2563eb' // blue-600
    default:
      return '#4b5563' // gray-600
  }
}

/**
 * Get color for different component states
 */
export function getStateColor(state: string): string {
  switch (state.toLowerCase()) {
    case 'running':
    case 'active':
    case 'on':
    case 'open':
      return '#10b981' // emerald-500
    case 'stopped':
    case 'inactive':
    case 'off':
    case 'closed':
      return '#6b7280' // gray-500
    case 'fault':
    case 'error':
    case 'alarm':
      return '#ef4444' // red-500
    case 'warning':
      return '#f59e0b' // amber-500
    case 'manual':
      return '#8b5cf6' // violet-500
    case 'auto':
      return '#3b82f6' // blue-500
    default:
      return '#6b7280' // gray-500
  }
}

/**
 * Get color for quality codes
 */
export function getQualityColor(quality: string): string {
  switch (quality.toLowerCase()) {
    case 'good':
      return '#10b981' // emerald-500
    case 'uncertain':
      return '#f59e0b' // amber-500
    case 'bad':
      return '#ef4444' // red-500
    default:
      return '#6b7280' // gray-500
  }
}

/**
 * Convert hex color to rgba with opacity
 */
export function hexToRgba(hex: string, opacity: number): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  if (!result) return hex
  
  const r = parseInt(result[1], 16)
  const g = parseInt(result[2], 16)
  const b = parseInt(result[3], 16)
  
  return `rgba(${r}, ${g}, ${b}, ${opacity})`
}

/**
 * Get a contrasting text color (black or white) based on background
 */
export function getContrastColor(bgColor: string): string {
  // Convert hex to RGB
  const hex = bgColor.replace('#', '')
  const r = parseInt(hex.substr(0, 2), 16)
  const g = parseInt(hex.substr(2, 2), 16)
  const b = parseInt(hex.substr(4, 2), 16)
  
  // Calculate luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255
  
  return luminance > 0.5 ? '#000000' : '#ffffff'
}

export function getTypeColor(type: string): string {
  switch (type) {
    case 'bool':
      return '#22c55e' // green
    case 'int':
      return '#f59e0b' // amber
    case 'float':
      return '#3b82f6' // blue
    default:
      return '#6b7280' // gray
  }
}

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
    default:
      return '#f3f4f6' // gray-100
  }
}

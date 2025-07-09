// src/types/hmi.ts

export type HMIComponentType =
  | 'tank'
  | 'pump'
  | 'valve'
  | 'gauge'
  | 'trend'
  | 'text'
  | 'button'
  | 'pipe'
  | 'motor'
  | 'indicator'
  | 'slider'
  | 'shape'
  | 'image'
  | 'group'
  | 'heat-exchanger'
  | 'conveyor'
  | 'mixer'
  
export interface ConnectionInfo {
  status: 'connected' | 'disconnected' | 'connecting' | 'error'
  latency: number
  uptime: number
  lastError?: string
  messageRate: number
  reconnectAttempts: number
}

export interface HMIComponent {
  id: string
  type: HMIComponentType
  position: { x: number; y: number }
  size: { width: number; height: number }
  rotation: number
  bindings: SignalBinding[]
  animations: Animation[]
  interactions: Interaction[]
  style: ComponentStyle
  properties: Record<string, any>
  locked?: boolean
  visible?: boolean
  layer?: number
  groupId?: string
}

export interface SignalBinding {
  property: string
  signal: string
  transform?: string // JavaScript expression for value transformation
  format?: string // Display format (for text/numbers)
}

export interface Animation {
  id: string
  property: string
  trigger: {
    type: 'signal' | 'time' | 'always'
    signal?: string
    condition?: string
    value?: any
  }
  animation: {
    from: any
    to: any
    duration: number
    easing: 'linear' | 'easeIn' | 'easeOut' | 'easeInOut'
    repeat?: boolean
  }
}

export interface Interaction {
  id: string
  event: 'click' | 'doubleClick' | 'hover' | 'drag'
  action: {
    type: 'setSignal' | 'toggle' | 'increment' | 'decrement' | 'navigate' | 'script'
    signal?: string
    value?: any
    script?: string
    target?: string
  }
  confirm?: boolean
  confirmMessage?: string
}

export interface ComponentStyle {
  fill?: string
  stroke?: string
  strokeWidth?: number
  opacity?: number
  fontSize?: number
  fontFamily?: string
  fontWeight?: string
  textAlign?: 'left' | 'center' | 'right'
  backgroundColor?: string
  borderRadius?: number
  shadow?: boolean
  // Additional style properties
  needleColor?: string
  textColor?: string
}

export interface HMIDisplay {
  id: string
  name: string
  size: { width: number; height: number }
  components: HMIComponent[]
  background?: string
  grid?: {
    show: boolean
    size: number
    snap: boolean
  }
  createdAt: string
  updatedAt: string
  tags?: string[]
}

// Component-specific property types
export interface TankProperties {
  maxLevel: number
  currentLevel: number
  alarmHigh: number
  alarmLow: number
  showLabel: boolean
  units: string
  liquidColor?: string
  showWaveAnimation?: boolean
  // Additional properties
  fillColor?: string
  label?: string
  isMetric?: boolean
}

export interface PumpProperties {
  running: boolean
  fault: boolean
  speed: number
  showStatus: boolean
  runAnimation?: boolean
}

export interface ValveProperties {
  open: boolean
  fault: boolean
  position: number
  valveType?: 'gate' | 'ball' | 'butterfly' | 'control'
  showPosition?: boolean
}

export interface GaugeProperties {
  min: number
  max: number
  value: number
  units: string
  showScale?: boolean
  majorTicks?: number
  minorTicks?: number
  ranges?: Array<{
    start: number
    end: number
    color: string
  }>
  // Additional properties
  label?: string
  showDigital?: boolean
  alarmLow?: number
  alarmHigh?: number
  warningLow?: number
  warningHigh?: number
}

export interface TrendProperties {
  signals: string[]
  timeRange: string
  yMin: number
  yMax: number
  showGrid: boolean
  showLegend: boolean
  updateInterval?: number
  maxPoints?: number
}

export interface ButtonProperties {
  text: string
  action: 'momentary' | 'toggle' | 'set'
  confirmRequired: boolean
  confirmMessage?: string
  activeColor?: string
  inactiveColor?: string
}

// Real-time data types
export interface SignalUpdate {
  signal: string
  value: any
  timestamp: number
  quality?: 'good' | 'bad' | 'uncertain'
}

export interface TrendData {
  signal: string
  data: Array<{
    timestamp: number
    value: number
  }>
}

// Export/deployment types
export interface HMIExportOptions {
  format: 'standalone' | 'embedded' | 'native'
  includeRuntime: boolean
  minified: boolean
  offlineSupport: boolean
  authentication?: {
    type: 'none' | 'basic' | 'jwt'
    endpoint?: string
  }
}

// Enhanced component bindings with expressions and validation
export interface EnhancedBinding extends SignalBinding {
  /** Optional JavaScript expression for computed bindings */
  expression?: string
  /** Input validation rules */
  validation?: {
    min?: number
    max?: number
    pattern?: string
  }
  /** Throttle updates (ms) */
  throttle?: number
  /** Conditional visibility expression */
  visibilityCondition?: string
}

// Component template definitions
export interface ComponentTemplate {
  name: string
  description: string
  thumbnail?: string
  component: Partial<HMIComponent>
  defaultBindings?: SignalBinding[]
}

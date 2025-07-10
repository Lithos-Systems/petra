import {
  FaToggleOn,
  FaClock,
  FaGreaterThan,
  FaLessThan,
  FaEquals,
  FaExclamation,
  FaCogs,
  FaChartLine,
  FaTimes,
  FaPlus,
  FaDivide,
} from 'react-icons/fa'
import { IconType } from 'react-icons'

/* ------------------------------------------------------------------ */
/* Pick an icon for each block type                                    */
/* ------------------------------------------------------------------ */
export function getBlockIcon(blockType: string): IconType {
  switch (blockType) {
    case 'AND':
      return FaToggleOn
    case 'OR':
      return FaToggleOn
    case 'NOT':
      return FaExclamation
    case 'GT':
      return FaGreaterThan
    case 'LT':
      return FaLessThan
    case 'EQ':
      return FaEquals
    case 'TON':
    case 'TOF':
      return FaClock
    case 'R_TRIG':
    case 'F_TRIG':
      return FaChartLine
    case 'SR_LATCH':
      return FaToggleOn
    case 'COUNTER':
      return FaPlus
    case 'MULTIPLY':
      return FaTimes
    case 'DIVIDE':
      return FaDivide
    case 'DATA_GENERATOR':
      return FaChartLine
    default:
      return FaCogs
  }
}

/* ------------------------------------------------------------------ */
/* Master list for <select> menus, grouped by category                 */
/* ------------------------------------------------------------------ */
export const BLOCK_TYPES = [
  { value: 'AND',           label: 'AND Gate',        category: 'Logic' },
  { value: 'OR',            label: 'OR Gate',         category: 'Logic' },
  { value: 'NOT',           label: 'NOT Gate',        category: 'Logic' },
  { value: 'GT',            label: 'Greater Than',    category: 'Comparison' },
  { value: 'LT',            label: 'Less Than',       category: 'Comparison' },
  { value: 'EQ',            label: 'Equal',           category: 'Comparison' },
  { value: 'TON',           label: 'Timer On Delay',  category: 'Timer' },
  { value: 'TOF',           label: 'Timer Off Delay', category: 'Timer' },
  { value: 'R_TRIG',        label: 'Rising Edge',     category: 'Edge' },
  { value: 'F_TRIG',        label: 'Falling Edge',    category: 'Edge' },
  { value: 'SR_LATCH',      label: 'SR Latch',        category: 'Memory' },
  { value: 'COUNTER',       label: 'Counter',         category: 'Counter' },
  { value: 'MULTIPLY',      label: 'Multiply',        category: 'Math' },
  { value: 'DIVIDE',        label: 'Divide',          category: 'Math' },
  { value: 'DATA_GENERATOR',label: 'Data Generator',  category: 'Generator' },
] as const

// Modern ISA-101 compliant block graphics
export function renderBlockGraphic(blockType: string) {
  const style = {
    stroke: '#404040',
    strokeWidth: 2,
    fill: 'none'
  }

  switch (blockType) {
    case 'AND':
      return (
        <svg width="40" height="30" viewBox="0 0 40 30">
          <path d={
            `M 5,5 L 20,5 Q 35,5 35,15 Q 35,25 20,25 L 5,25 Z`
          } {...style} />
          <text x="20" y="18" textAnchor="middle" fontSize="10" fill="#000">
            &amp;
          </text>
        </svg>
      )
    case 'PID':
      return (
        <svg width="40" height="30" viewBox="0 0 40 30">
          <rect x="5" y="5" width="30" height="20" rx="2" {...style} />
          <text x="20" y="18" textAnchor="middle" fontSize="8" fill="#000">
            PID
          </text>
        </svg>
      )
    default:
      return (
        <svg width="40" height="30" viewBox="0 0 40 30">
          <rect x="5" y="5" width="30" height="20" rx="2" {...style} />
          <text x="20" y="18" textAnchor="middle" fontSize="8" fill="#000">
            {blockType}
          </text>
        </svg>
      )
  }
}

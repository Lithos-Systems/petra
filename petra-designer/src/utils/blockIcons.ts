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

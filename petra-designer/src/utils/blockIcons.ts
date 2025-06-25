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

export function getBlockIcon(blockType: string): IconType {
  switch (blockType) {
    case 'AND':
      return FaToggleOn       // ⬅️ replaced missing FaAnd
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

/* unchanged BLOCK_TYPES array */

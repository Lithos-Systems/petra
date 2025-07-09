import ISA101TankComponent from './ISA101TankComponent'
import ISA101PumpComponent from './ISA101PumpComponent'
import ISA101ValveComponent from './ISA101ValveComponent'

export default function ISA101ComponentRenderer({ component, ...props }) {
  switch (component.type) {
    case 'tank':
      return <ISA101TankComponent {...props} />
    case 'pump':
      return <ISA101PumpComponent {...props} />
    case 'valve':
      return <ISA101ValveComponent {...props} />
    default:
      return null
  }
}

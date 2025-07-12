import React from 'react'
import type { HMIComponent } from '@/types/hmi'
import ISA101TankComponent from './ISA101TankComponent'
import ISA101PumpComponent from './ISA101PumpComponent'
import ISA101ValveComponent from './ISA101ValveComponent'
import ISA101GaugeComponent from './ISA101GaugeComponent'
import ISA101TrendComponent from './ISA101TrendComponent'
import ISA101ButtonComponent from './ISA101ButtonComponent'
import ISA101IndicatorComponent from './ISA101IndicatorComponent'
import ISA101TextComponent from './ISA101TextComponent'
import ISA101MotorComponent from './ISA101MotorComponent'
import ISA101HeatExchangerComponent from './ISA101HeatExchangerComponent'
import ISA101ConveyorComponent from './ISA101ConveyorComponent'
import ISA101MixerComponent from './ISA101MixerComponent'
import ISA101PipeComponent from './ISA101PipeComponent'
import ISA101ShapeComponent from './ISA101ShapeComponent'

interface ISA101ComponentRendererProps {
  component: HMIComponent
  isSelected: boolean
  onSelect: () => void
  onUpdate: (updates: Partial<HMIComponent>) => void
}

export default function ISA101ComponentRenderer({
  component,
  isSelected,
  onSelect,
  onUpdate,
}: ISA101ComponentRendererProps) {
  // Map standard properties to ISA-101 compliant properties
  const mapToISA101Properties = (type: string, properties: any) => {
    switch (type) {
      case 'tank':
        return {
          tagName: properties.tagName || 'TK-101',
          currentLevel: properties.currentLevel || properties.level || 50,
          levelUnits: properties.units || '%',
          maxLevel: properties.maxLevel || 100,
          minLevel: properties.minLevel || 0,
          alarmHigh: properties.alarmHigh || 80,
          alarmLow: properties.alarmLow || 20,
          criticalHigh: properties.criticalHigh || 95,
          criticalLow: properties.criticalLow || 5,
          alarmState: properties.alarmState || 'none',
          showTrend: properties.showTrend !== false,
          showAlarmLimits: properties.showAlarmLimits !== false,
          materialType: properties.materialType || 'water',
        }
      case 'pump':
        return {
          tagName: properties.tagName || 'P-101',
          status: properties.running ? 'running' : properties.fault ? 'fault' : 'stopped',
          flowRate: properties.flowRate || 0,
          flowUnits: properties.flowUnits || 'GPM',
          dischargePressure: properties.pressure || 0,
          pressureUnits: properties.pressureUnits || 'PSI',
          speed: properties.speed || 0,
          controlMode: properties.controlMode || 'auto',
          interlocked: properties.interlocked || false,
          showDetailedStatus: properties.showDetailedStatus !== false,
          showFlowDirection: properties.showFlowDirection !== false,
        }
      case 'valve':
        return {
          tagName: properties.tagName || 'V-101',
          position: properties.position || (properties.open ? 100 : 0),
          status: properties.transitioning ? 'transitioning' : 
                  properties.fault ? 'fault' :
                  properties.position > 50 ? 'open' : 'closed',
          valveType: properties.valveType || 'gate',
          controlMode: properties.controlMode || 'auto',
          interlocked: properties.interlocked || false,
          showPosition: properties.showPosition !== false,
          orientation: properties.orientation || 'horizontal',
        }
      case 'gauge':
        return {
          tagName: properties.tagName || 'PI-101',
          currentValue: properties.value || 0,
          units: properties.units || 'PSI',
          minValue: properties.min || 0,
          maxValue: properties.max || 100,
          alarmHigh: properties.alarmHigh || 80,
          alarmLow: properties.alarmLow || 20,
          showDigitalValue: true,
          gaugeType: 'pressure',
        }
      case 'motor':
        return {
          tagName: properties.tagName || 'M-101',
          running: properties.running || false,
          speed: properties.speed || 0,
          current: properties.current || 0,
          temperature: properties.temperature || 0,
          fault: properties.fault || false,
          controlMode: properties.controlMode || 'auto',
        }
      default:
        return properties
    }
  }

  const isa101Props = {
    x: component.position.x,
    y: component.position.y,
    width: component.size.width,
    height: component.size.height,
    properties: mapToISA101Properties(component.type, component.properties),
    style: component.style,
    selected: isSelected,
    onContextMenu: (e: any) => {
      e.evt.preventDefault()
      onSelect()
    },
    onClick: onSelect,
    draggable: true,
    onDragEnd: (e: any) => {
      const node = e.target
      onUpdate({
        position: {
          x: node.x(),
          y: node.y(),
        },
      })
      node.getLayer()?.batchDraw()
    },
  }

  switch (component.type) {
    case 'tank':
      return <ISA101TankComponent {...isa101Props} />
    case 'pump':
      return <ISA101PumpComponent {...isa101Props} />
    case 'valve':
      return <ISA101ValveComponent {...isa101Props} />
    case 'gauge':
      return <ISA101GaugeComponent {...isa101Props} />
    case 'trend':
      return <ISA101TrendComponent {...isa101Props} />
    case 'button':
      return <ISA101ButtonComponent {...isa101Props} />
    case 'indicator':
      return <ISA101IndicatorComponent {...isa101Props} />
    case 'text':
      return <ISA101TextComponent {...isa101Props} />
    case 'motor':
      return <ISA101MotorComponent {...isa101Props} />
    case 'heat-exchanger':
      return <ISA101HeatExchangerComponent {...isa101Props} />
    case 'conveyor':
      return <ISA101ConveyorComponent {...isa101Props} />
    case 'mixer':
      return <ISA101MixerComponent {...isa101Props} />
    case 'pipe':
      return <ISA101PipeComponent {...isa101Props} />
    case 'shape':
      return <ISA101ShapeComponent {...isa101Props} />
    case 'slider':
      // For now, render as a horizontal gauge
      return <ISA101GaugeComponent {...isa101Props} />
    default:
      console.warn(`Unknown component type: ${component.type}`)
      // Fallback to a generic shape
      return <ISA101ShapeComponent {...isa101Props} />
  }
}

import { Group, Circle, Rect, Text } from 'react-konva'
import { useRef, useState, useEffect } from 'react'

const ISA101Colors = {
  equipmentOutline: '#000000',
  indicatorOn: '#00FF00',
  indicatorOff: '#808080',
  alarmRed: '#FF0000',
  alarmYellow: '#FFFF00',
  processValue: '#000000',
  background: '#FFFFFF',
}

interface ISA101IndicatorProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName?: string
    on: boolean
    onColor?: string
    offColor?: string
    label?: string
    shape?: 'circle' | 'square'
    blink?: boolean
    blinkRate?: number // ms
  }
  style?: any
  selected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
  onClick?: () => void
  onContextMenu?: (e: any) => void
}

export default function ISA101IndicatorComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected,
  draggable,
  onDragEnd,
  onDragStart,
  onClick,
  onContextMenu,
  ...restProps
}: ISA101IndicatorProps) {
  const [isVisible, setIsVisible] = useState(true)
  const groupRef = useRef<any>()

  useEffect(() => {
    let interval: any
    if (properties.blink && properties.on) {
      interval = setInterval(() => {
        setIsVisible(prev => !prev)
      }, properties.blinkRate || 500)
    } else {
      setIsVisible(true)
    }
    
    return () => {
      if (interval) clearInterval(interval)
    }
  }, [properties.blink, properties.on, properties.blinkRate])

  const getIndicatorColor = () => {
    if (!isVisible) return ISA101Colors.indicatorOff
    return properties.on 
      ? (properties.onColor || ISA101Colors.indicatorOn)
      : (properties.offColor || ISA101Colors.indicatorOff)
  }

  const indicatorSize = Math.min(width, height) * 0.8

  return (
    <Group
      ref={groupRef}
      x={x}
      y={y}
      draggable={draggable}
      onDragEnd={onDragEnd}
      onDragStart={(e) => {
        e.target.moveToTop()
        onDragStart?.(e)
      }}
      onClick={onClick}
      onContextMenu={onContextMenu}
      {...restProps}
    >
      {/* Selection indicator */}
      {selected && (
        <Rect
          x={-5}
          y={-5}
          width={width + 10}
          height={height + 10}
          stroke="#0080FF"
          strokeWidth={2}
          dash={[5, 5]}
          fill="transparent"
        />
      )}

      {/* Indicator shape */}
      {properties.shape === 'square' ? (
        <Rect
          x={(width - indicatorSize) / 2}
          y={(height - indicatorSize) / 2}
          width={indicatorSize}
          height={indicatorSize}
          fill={getIndicatorColor()}
          stroke={ISA101Colors.equipmentOutline}
          strokeWidth={2}
          cornerRadius={2}
        />
      ) : (
        <Circle
          x={width / 2}
          y={height / 2}
          radius={indicatorSize / 2}
          fill={getIndicatorColor()}
          stroke={ISA101Colors.equipmentOutline}
          strokeWidth={2}
        />
      )}

      {/* Tag name */}
      {properties.tagName && (
        <Text
          x={0}
          y={-15}
          width={width}
          text={properties.tagName}
          fontSize={10}
          fill={ISA101Colors.processValue}
          align="center"
        />
      )}

      {/* Label */}
      {properties.label && (
        <Text
          x={0}
          y={height + 2}
          width={width}
          text={properties.label}
          fontSize={10}
          fill={ISA101Colors.processValue}
          align="center"
        />
      )}
    </Group>
  )
}

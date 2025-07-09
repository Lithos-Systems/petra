// @ts-nocheck
import { Group, Rect, Circle, Line, RegularPolygon } from 'react-konva'
import { useRef } from 'react'

const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#E6E6E6',
  background: '#FFFFFF',
}

interface ISA101ShapeProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    shapeType: 'rectangle' | 'circle' | 'triangle' | 'hexagon' | 'line'
    fill?: string
    stroke?: string
    strokeWidth?: number
    cornerRadius?: number
    opacity?: number
  }
  style?: any
  selected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
  onClick?: () => void
  onContextMenu?: (e: any) => void
}

export default function ISA101ShapeComponent({
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
}: ISA101ShapeProps) {
  const groupRef = useRef<any>()

  const renderShape = () => {
    const commonProps = {
      fill: properties.fill || 'transparent',
      stroke: properties.stroke || ISA101Colors.equipmentOutline,
      strokeWidth: properties.strokeWidth || 2,
      opacity: properties.opacity || 1,
    }

    switch (properties.shapeType) {
      case 'circle':
        return (
          <Circle
            x={width / 2}
            y={height / 2}
            radius={Math.min(width, height) / 2}
            {...commonProps}
          />
        )
      
      case 'triangle':
        return (
          <RegularPolygon
            x={width / 2}
            y={height / 2}
            sides={3}
            radius={Math.min(width, height) / 2}
            {...commonProps}
          />
        )
      
      case 'hexagon':
        return (
          <RegularPolygon
            x={width / 2}
            y={height / 2}
            sides={6}
            radius={Math.min(width, height) / 2}
            {...commonProps}
          />
        )
      
      case 'line':
        return (
          <Line
            points={[0, height / 2, width, height / 2]}
            stroke={properties.stroke || ISA101Colors.equipmentOutline}
            strokeWidth={properties.strokeWidth || 2}
            opacity={properties.opacity || 1}
          />
        )
      
      case 'rectangle':
      default:
        return (
          <Rect
            x={0}
            y={0}
            width={width}
            height={height}
            cornerRadius={properties.cornerRadius || 0}
            {...commonProps}
          />
        )
    }
  }

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

      {/* Render the shape */}
      {renderShape()}
    </Group>
  )
}

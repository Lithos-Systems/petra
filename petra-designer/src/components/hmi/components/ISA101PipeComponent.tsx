// @ts-nocheck
import { Group, Line, Rect, Arrow } from 'react-konva'
import { useRef, useEffect, useState } from 'react'

const ISA101Colors = {
  processLine: '#000000',
  flowActive: '#00FF00',
  flowInactive: '#808080',
}

interface ISA101PipeProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    points?: number[] // [x1, y1, x2, y2, ...]
    flowAnimation?: boolean
    flowDirection?: 'forward' | 'reverse' | 'none'
    pipeSize?: number
    showArrows?: boolean
    flowing?: boolean
  }
  style?: any
  selected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
  onClick?: () => void
  onContextMenu?: (e: any) => void
}

export default function ISA101PipeComponent({
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
}: ISA101PipeProps) {
  const [dashOffset, setDashOffset] = useState(0)
  const groupRef = useRef<any>()

  useEffect(() => {
    let animationId: any
    if (properties.flowAnimation && properties.flowing) {
      const animate = () => {
        setDashOffset(prev => (prev - 2) % 20)
        animationId = requestAnimationFrame(animate)
      }
      animationId = requestAnimationFrame(animate)
    }
    
    return () => {
      if (animationId) cancelAnimationFrame(animationId)
    }
  }, [properties.flowAnimation, properties.flowing])

  // Default to horizontal pipe if no points specified
  const points = properties.points || [0, height/2, width, height/2]
  
  // Calculate arrow positions for flow direction
  const getArrowPoints = () => {
    if (!properties.showArrows || properties.flowDirection === 'none') return []
    
    const arrows = []
    const numSegments = Math.floor(points.length / 2) - 1
    
    for (let i = 0; i < numSegments; i++) {
      const x1 = points[i * 2]
      const y1 = points[i * 2 + 1]
      const x2 = points[(i + 1) * 2]
      const y2 = points[(i + 1) * 2 + 1]
      
      const midX = (x1 + x2) / 2
      const midY = (y1 + y2) / 2
      
      arrows.push({
        x: midX,
        y: midY,
        rotation: Math.atan2(y2 - y1, x2 - x1) * 180 / Math.PI
      })
    }
    
    return arrows
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

      {/* Main pipe line */}
      <Line
        points={points}
        stroke={properties.flowing ? ISA101Colors.flowActive : ISA101Colors.processLine}
        strokeWidth={properties.pipeSize || 6}
        lineCap="round"
        lineJoin="round"
      />

      {/* Flow animation overlay */}
      {properties.flowAnimation && properties.flowing && (
        <Line
          points={points}
          stroke="#FFFFFF"
          strokeWidth={(properties.pipeSize || 6) * 0.3}
          lineCap="round"
          lineJoin="round"
          dash={[10, 10]}
          dashOffset={dashOffset}
          opacity={0.6}
        />
      )}

      {/* Flow direction arrows */}
      {getArrowPoints().map((arrow, idx) => (
        <Arrow
          key={idx}
          x={arrow.x}
          y={arrow.y}
          rotation={properties.flowDirection === 'reverse' ? arrow.rotation + 180 : arrow.rotation}
          points={[0, 0, 10, 0]}
          pointerLength={6}
          pointerWidth={6}
          fill={properties.flowing ? ISA101Colors.flowActive : ISA101Colors.processLine}
          stroke={properties.flowing ? ISA101Colors.flowActive : ISA101Colors.processLine}
          strokeWidth={1}
        />
      ))}
    </Group>
  )
}

import { Group, Rect, Line, Path, Text } from 'react-konva'
import type { ValveProperties } from '@/types/hmi'

interface ValveComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: ValveProperties
  style: any
  [key: string]: any
}

export default function ValveComponent({
  x, y, width, height, properties, style, ...rest
}: ValveComponentProps) {
  const isOpen = properties.open
  const centerX = width / 2
  const centerY = height / 2
  
  return (
    <Group x={x} y={y} {...rest}>
      {/* Valve body */}
      <Path
        data={`M 0 ${centerY - 10} L ${centerX - 15} ${centerY - 10} L ${centerX} ${centerY - 20} L ${centerX + 15} ${centerY - 10} L ${width} ${centerY - 10} L ${width} ${centerY + 10} L ${centerX + 15} ${centerY + 10} L ${centerX} ${centerY + 20} L ${centerX - 15} ${centerY + 10} L 0 ${centerY + 10} Z`}
        fill={properties.fault ? '#ff4444' : style.fill || '#666666'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
      />
      
      {/* Valve stem */}
      <Rect
        x={centerX - 5}
        y={centerY - 30}
        width={10}
        height={30}
        fill={style.stroke || '#333333'}
        rotation={isOpen ? 0 : 90}
        offsetX={5}
        offsetY={15}
      />
      
      {/* Position indicator */}
      {properties.showPosition && (
        <Text
          x={0}
          y={height + 5}
          width={width}
          text={`${properties.position}%`}
          fontSize={11}
          fill="#666666"
          align="center"
        />
      )}
    </Group>
  )
}

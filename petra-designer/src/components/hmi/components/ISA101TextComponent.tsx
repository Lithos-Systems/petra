import { Group, Text, Rect } from 'react-konva'
import { useRef } from 'react'

const ISA101Colors = {
  processValue: '#000000',
  textSecondary: '#666666',
  background: '#FFFFFF',
  equipmentOutline: '#000000',
}

interface ISA101TextProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    text: string
    fontSize?: number
    fontWeight?: string
    textAlign?: 'left' | 'center' | 'right'
    verticalAlign?: 'top' | 'middle' | 'bottom'
    fontFamily?: string
    color?: string
    backgroundColor?: string
    showBorder?: boolean
  }
  style?: any
  selected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
  onClick?: () => void
  onContextMenu?: (e: any) => void
}

export default function ISA101TextComponent({
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
}: ISA101TextProps) {
  const groupRef = useRef<any>()

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

      {/* Background */}
      {(properties.backgroundColor || properties.showBorder) && (
        <Rect
          x={0}
          y={0}
          width={width}
          height={height}
          fill={properties.backgroundColor || 'transparent'}
          stroke={properties.showBorder ? ISA101Colors.equipmentOutline : 'transparent'}
          strokeWidth={1}
        />
      )}

      {/* Text */}
      <Text
        x={0}
        y={0}
        width={width}
        height={height}
        text={properties.text}
        fontSize={properties.fontSize || 12}
        fontFamily={properties.fontFamily || 'Arial'}
        fontStyle={properties.fontWeight || 'normal'}
        fill={properties.color || ISA101Colors.processValue}
        align={properties.textAlign || 'left'}
        verticalAlign={properties.verticalAlign || 'top'}
        wrap="word"
        ellipsis={true}
      />
    </Group>
  )
}

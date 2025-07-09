// @ts-nocheck
import { Group, Rect, Text } from 'react-konva'

export default function ISA101ButtonComponent({ x, y, width, height, properties, ...props }: any) {
  return (
    <Group x={x} y={y} {...props}>
      <Rect
        width={width}
        height={height}
        fill="#FFFFFF"
        stroke="#000000"
        strokeWidth={2}
        shadowBlur={properties.pressed ? 0 : 2}
      />
      <Text
        width={width}
        height={height}
        text={properties.text || 'Button'}
        fontSize={14}
        fontFamily="Arial"
        fill="#000000"
        align="center"
        verticalAlign="middle"
      />
    </Group>
  )
}

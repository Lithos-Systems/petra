// @ts-nocheck
import { Group, Text } from 'react-konva'

export default function ISA101TextComponent({ x, y, width, height, properties, ...props }: any) {
  return (
    <Group x={x} y={y} {...props}>
      <Text
        x={0}
        y={0}
        width={width}
        height={height}
        text={properties.text || 'Text'}
        fontSize={properties.fontSize || 16}
        fontFamily="Arial"
        fill="#000"
        align={properties.align || 'center'}
        verticalAlign="middle"
      />
    </Group>
  )
}

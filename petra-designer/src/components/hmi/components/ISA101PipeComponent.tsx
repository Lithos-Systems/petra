// @ts-nocheck
import { Group, Rect } from 'react-konva'

export default function ISA101PipeComponent({ x, y, width, height, properties, ...props }: any) {
  return (
    <Group x={x} y={y} {...props}>
      <Rect
        x={0}
        y={height / 2 - 10}
        width={width}
        height={20}
        fill="#666"
        stroke="#333"
        strokeWidth={2}
      />
    </Group>
  )
}

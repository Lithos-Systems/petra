// @ts-nocheck
import { Group, Rect } from 'react-konva'

export default function ISA101ShapeComponent({ x, y, width, height, style = {}, ...props }: any) {
  return (
    <Group x={x} y={y} {...props}>
      <Rect
        x={0}
        y={0}
        width={width}
        height={height}
        fill={style.fill || '#cccccc'}
        stroke={style.stroke || '#333'}
        strokeWidth={style.strokeWidth || 2}
        cornerRadius={style.borderRadius || 0}
      />
    </Group>
  )
}

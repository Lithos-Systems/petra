import { Group, Rect, Line } from 'react-konva'

export default function ISA101HeatExchangerComponent({ x, y, width, height, properties, ...props }: any) {
  return (
    <Group x={x} y={y} {...props}>
      <Rect
        width={width}
        height={height}
        fill="#ffffff"
        stroke="#000"
        strokeWidth={2}
      />
      {Array.from({ length: 4 }).map((_, i) => (
        <Line
          key={i}
          points={[10 + i * (width - 20) / 3, 5, 10 + i * (width - 20) / 3, height - 5]}
          stroke="#000"
          strokeWidth={2}
        />
      ))}
    </Group>
  )
}

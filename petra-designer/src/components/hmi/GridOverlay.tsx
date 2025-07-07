import { Line, Group } from 'react-konva'

interface GridOverlayProps {
  width: number
  height: number
  gridSize: number
}

export default function GridOverlay({ width, height, gridSize }: GridOverlayProps) {
  const lines = []

  // Vertical lines
  for (let x = 0; x <= width; x += gridSize) {
    lines.push(
      <Line
        key={`v-${x}`}
        points={[x, 0, x, height]}
        stroke="#e5e7eb"
        strokeWidth={1}
        listening={false}
      />
    )
  }

  // Horizontal lines
  for (let y = 0; y <= height; y += gridSize) {
    lines.push(
      <Line
        key={`h-${y}`}
        points={[0, y, width, y]}
        stroke="#e5e7eb"
        strokeWidth={1}
        listening={false}
      />
    )
  }

  // Major grid lines every 5 cells
  const majorGridSize = gridSize * 5
  
  // Major vertical lines
  for (let x = 0; x <= width; x += majorGridSize) {
    lines.push(
      <Line
        key={`mv-${x}`}
        points={[x, 0, x, height]}
        stroke="#d1d5db"
        strokeWidth={1}
        listening={false}
      />
    )
  }

  // Major horizontal lines
  for (let y = 0; y <= height; y += majorGridSize) {
    lines.push(
      <Line
        key={`mh-${y}`}
        points={[0, y, width, y]}
        stroke="#d1d5db"
        strokeWidth={1}
        listening={false}
      />
    )
  }

  return <Group listening={false}>{lines}</Group>
}

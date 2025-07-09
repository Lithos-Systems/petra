import { useState, useEffect } from 'react'

interface ResizablePanelProps {
  defaultWidth: number
  minWidth: number
  maxWidth: number
  children: React.ReactNode
  className?: string
}

export function ResizablePanel({
  defaultWidth,
  minWidth,
  maxWidth,
  children,
  className,
}: ResizablePanelProps) {
  const [width, setWidth] = useState(defaultWidth)
  const [isResizing, setIsResizing] = useState(false)

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsResizing(true)
    e.preventDefault()
  }

  useEffect(() => {
    if (!isResizing) return

    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = window.innerWidth - e.clientX
      setWidth(Math.max(minWidth, Math.min(maxWidth, newWidth)))
    }

    const handleMouseUp = () => {
      setIsResizing(false)
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)

    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isResizing, minWidth, maxWidth])

  return (
    <div className={`relative ${className || ''}`} style={{ width }}>
      <div
        className="absolute left-0 top-0 bottom-0 w-1 cursor-ew-resize hover:bg-blue-500 transition-colors"
        onMouseDown={handleMouseDown}
        style={{ left: -2 }}
      />
      {children}
    </div>
  )
}
export default ResizablePanel

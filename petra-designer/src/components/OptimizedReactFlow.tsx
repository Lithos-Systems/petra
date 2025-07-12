// petra-designer/src/components/OptimizedReactFlow.tsx
import React, { useMemo, useRef, useEffect } from 'react'
import { 
  ReactFlow, 
  Background, 
  Controls, 
  MiniMap, 
  BackgroundVariant,
  getBezierPath,
  EdgeProps,
  BaseEdge,
  ReactFlowInstance
} from '@xyflow/react'
import { OptimizedBlockNode } from '@/nodes/OptimizedBlockNode'
import { nodeTypes as baseNodeTypes } from '@/nodes'

// Memoized edge component to prevent re-renders
const CustomEdge = React.memo(({ 
  id, 
  sourceX, 
  sourceY, 
  targetX, 
  targetY, 
  sourcePosition, 
  targetPosition, 
  selected
}: EdgeProps) => {
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })

  return (
    <BaseEdge 
      id={id} 
      path={edgePath} 
      style={{
        stroke: selected ? '#000080' : '#000000',
        strokeWidth: selected ? 3 : 2,
      }}
    />
  )
})

// Static node types to prevent recreation
const nodeTypes = { 
  ...baseNodeTypes, 
  block: OptimizedBlockNode 
} as const

// Static edge types to prevent recreation
const edgeTypes = {
  default: CustomEdge,
} as const

export const OptimizedReactFlow = React.memo(({
  nodes,
  edges,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onNodeClick,
  onPaneClick,
  onDrop,
  onDragOver,
  onEdgeClick,
  className
}: any) => {
  const reactFlowInstance = useRef<ReactFlowInstance | null>(null)
  
  // Clean up event listeners on unmount
  useEffect(() => {
    return () => {
      // Clear any pending animations/renders
      if (reactFlowInstance.current) {
        reactFlowInstance.current = null
      }
    }
  }, [])

  // Memoize all props to prevent unnecessary re-renders
  const reactFlowProps = useMemo(() => ({
    nodes,
    edges,
    nodeTypes,
    edgeTypes,
    onNodesChange,
    onEdgesChange,
    onConnect,
    onNodeClick,
    onPaneClick,
    onDrop,
    onDragOver,
    onEdgeClick,
    onInit: (instance: ReactFlowInstance) => {
      reactFlowInstance.current = instance
    },
    // Performance optimizations
    fitView: false,
    snapToGrid: true,
    snapGrid: [10, 10] as [number, number],
    nodesDraggable: true,
    nodesConnectable: true,
    elementsSelectable: true,
    panOnDrag: true,
    panOnScroll: false,
    zoomOnScroll: true,
    zoomOnPinch: true,
    selectNodesOnDrag: false,
    // Reduce zoom range to improve performance
    maxZoom: 1.5,
    minZoom: 0.5,
    // Optimize pan/zoom speeds
    panOnScrollSpeed: 0.5,
    zoomOnScrollSpeed: 0.5,
    // Disable animations for better performance
    defaultEdgeOptions: {
      type: 'default',
      animated: false,
      style: { 
        strokeWidth: 2, 
        stroke: '#000000' 
      }
    },
    deleteKeyCode: ['Delete', 'Backspace'],
    // Improve render performance
    elevateNodesOnSelect: false,
    // Disable node extent to improve drag performance
    nodeExtent: undefined,
    translateExtent: undefined,
    // Set viewport for consistent initial view
    defaultViewport: { x: 0, y: 0, zoom: 1 }
  }), [
    nodes, 
    edges, 
    onNodesChange, 
    onEdgesChange, 
    onConnect, 
    onNodeClick, 
    onPaneClick, 
    onDrop, 
    onDragOver, 
    onEdgeClick
  ])

  return (
    <ReactFlow 
      {...reactFlowProps} 
      className={className}
    >
      <Background 
        variant={BackgroundVariant.Dots} 
        gap={20} 
        size={1} 
        color="#A0A0A0" 
      />
      <Controls
        showZoom={true}
        showFitView={true}
        showInteractive={false}
        position="top-left"
        style={{
          backgroundColor: '#E0E0E0',
          border: '1px solid #404040',
          borderRadius: 0
        }}
      />
      <MiniMap
        nodeStrokeWidth={2}
        nodeColor="#808080"
        position="bottom-left"
        style={{
          backgroundColor: '#E0E0E0',
          border: '1px solid #404040',
          borderRadius: 0
        }}
        pannable={false}
        zoomable={false}
        // Reduce minimap node count for performance
        nodeClassName="minimap-node"
      />
    </ReactFlow>
  )
}, (prevProps, nextProps) => {
  // Custom comparison to prevent unnecessary re-renders
  return (
    prevProps.nodes === nextProps.nodes &&
    prevProps.edges === nextProps.edges &&
    prevProps.className === nextProps.className
  )
})

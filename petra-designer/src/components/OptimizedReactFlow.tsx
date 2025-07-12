// petra-designer/src/components/OptimizedReactFlow.tsx
import React, { useMemo } from 'react'
import { 
  ReactFlow, 
  Background, 
  Controls, 
  MiniMap, 
  BackgroundVariant,
  getBezierPath,
  EdgeProps,
  BaseEdge
} from '@xyflow/react'
import { OptimizedBlockNode } from '@/nodes/OptimizedBlockNode'
import { nodeTypes as baseNodeTypes } from '@/nodes'

// Custom edge component with bezier curves
const CustomEdge = ({ 
  id, 
  sourceX, 
  sourceY, 
  targetX, 
  targetY, 
  sourcePosition, 
  targetPosition, 
  data,
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
    <>
      <BaseEdge 
        id={id} 
        path={edgePath} 
        style={{
          stroke: selected ? '#000080' : '#000000',
          strokeWidth: selected ? 3 : 2,
        }}
      />
    </>
  )
}

const nodeTypes = { ...baseNodeTypes, block: OptimizedBlockNode } as const
const edgeTypes = {
  default: CustomEdge,
}

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
    fitView: false,
    snapToGrid: true,
    snapGrid: [10, 10] as [number, number],
    defaultEdgeOptions: {
      type: 'default',
      animated: false,
      style: { strokeWidth: 2, stroke: '#000000' }
    },
    elementsSelectable: true,
    nodesConnectable: true,
    nodesDraggable: true,
    panOnDrag: true,
    panOnScroll: false,
    zoomOnScroll: true,
    zoomOnPinch: true,
    selectNodesOnDrag: false,
    deleteKeyCode: ['Delete', 'Backspace'],
    maxZoom: 1.5,
    minZoom: 0.5,
    panOnScrollSpeed: 0.5,
    zoomOnScrollSpeed: 0.5,
    viewport: { x: 0, y: 0, zoom: 1 }
  }), [nodes, edges, onNodesChange, onEdgesChange, onConnect, onNodeClick, onPaneClick, onDrop, onDragOver, onEdgeClick])

  return (
    <ReactFlow {...reactFlowProps} className={className}>
      <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#A0A0A0" />
      <Controls
        showZoom={true}
        showFitView={true}
        showInteractive={false}
        style={{
          backgroundColor: '#E0E0E0',
          border: '1px solid #404040',
          borderRadius: 0
        }}
      />
      <MiniMap
        nodeStrokeWidth={2}
        nodeColor="#808080"
        style={{
          backgroundColor: '#E0E0E0',
          border: '1px solid #404040',
          borderRadius: 0
        }}
        pannable={false}
        zoomable={false}
      />
    </ReactFlow>
  )
})

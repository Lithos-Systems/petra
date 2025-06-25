import { useCallback, DragEvent, useEffect } from 'react'
import { 
  ReactFlow, 
  Background, 
  Controls, 
  MiniMap, 
  ReactFlowProvider,
  Node,
  Edge,
  Connection
} from '@xyflow/react'
import { Toaster } from 'react-hot-toast'

// Wrap imports in try-catch to see which one fails
let useFlowStore: any;
let nodeTypes: any;
let Sidebar: any;
let PropertiesPanel: any;
let YamlPreview: any;
let Toolbar: any;
let ErrorBoundary: any;

try {
  const storeModule = require('./store/flowStore');
  useFlowStore = storeModule.useFlowStore;
  console.log('✓ flowStore loaded');
} catch (e) {
  console.error('Failed to load flowStore:', e);
}

try {
  const nodesModule = require('./nodes');
  nodeTypes = nodesModule.nodeTypes;
  console.log('✓ nodeTypes loaded');
} catch (e) {
  console.error('Failed to load nodeTypes:', e);
}

try {
  Sidebar = require('./components/Sidebar').default;
  console.log('✓ Sidebar loaded');
} catch (e) {
  console.error('Failed to load Sidebar:', e);
}

try {
  PropertiesPanel = require('./components/PropertiesPanel').default;
  console.log('✓ PropertiesPanel loaded');
} catch (e) {
  console.error('Failed to load PropertiesPanel:', e);
}

try {
  YamlPreview = require('./components/YamlPreview').default;
  console.log('✓ YamlPreview loaded');
} catch (e) {
  console.error('Failed to load YamlPreview:', e);
}

try {
  Toolbar = require('./components/Toolbar').default;
  console.log('✓ Toolbar loaded');
} catch (e) {
  console.error('Failed to load Toolbar:', e);
}

try {
  ErrorBoundary = require('./components/ErrorBoundary').ErrorBoundary;
  console.log('✓ ErrorBoundary loaded');
} catch (e) {
  console.error('Failed to load ErrorBoundary:', e);
}

function Flow() {
  // If useFlowStore didn't load, show error
  if (!useFlowStore) {
    return <div>Error: Could not load flow store</div>;
  }

  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    selectedNode,
    addNode,
    setSelectedNode,
  } = useFlowStore()

  useEffect(() => {
    console.log('ReactFlow mounted, nodes:', nodes.length, 'edges:', edges.length)
  }, [nodes, edges])

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()
  
      const type = event.dataTransfer.getData('application/reactflow')
      if (!type) return
  
      const reactFlowBounds = event.currentTarget.getBoundingClientRect()
      const position = {
        x: event.clientX - reactFlowBounds.left - 100,
        y: event.clientY - reactFlowBounds.top - 20,
      }
  
      addNode(type, position)
    },
    [addNode]
  )

  const onNodeClick = useCallback(
    (_: any, node: any) => {
      setSelectedNode(node)
    },
    [setSelectedNode]
  )

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Toaster position="top-right" />
      {Toolbar && <Toolbar />}
      
      <div className="flex-1 flex">
        {Sidebar && <Sidebar />}
        
        <div className="flex-1 relative">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onDrop={onDrop}
            onDragOver={onDragOver}
            nodeTypes={nodeTypes || {}}
            fitView
            className="bg-gray-50"
          >
            <Background variant="dots" gap={20} className="bg-gray-50" />
            <Controls />
            <MiniMap className="bg-white" />
          </ReactFlow>
        </div>
        
        <div className="flex">
          {selectedNode && PropertiesPanel && <PropertiesPanel />}
          {YamlPreview && <YamlPreview />}
        </div>
      </div>
    </div>
  )
}

function App() {
  if (ErrorBoundary) {
    return (
      <ErrorBoundary>
        <ReactFlowProvider>
          <Flow />
        </ReactFlowProvider>
      </ErrorBoundary>
    )
  }

  return (
    <ReactFlowProvider>
      <Flow />
    </ReactFlowProvider>
  )
}

export default App

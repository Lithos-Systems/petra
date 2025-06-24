import { ReactFlow, Background, Controls, MiniMap, Panel } from '@xyflow/react'
import { Toaster } from 'react-hot-toast'
import { useFlowStore } from './store/flowStore'
import { nodeTypes } from './nodes'
import Sidebar from './components/Sidebar'
import PropertiesPanel from './components/PropertiesPanel'
import YamlPreview from './components/YamlPreview'
import Toolbar from './components/Toolbar'

function App() {
  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    selectedNode,
  } = useFlowStore()

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Toaster position="top-right" />
      <Toolbar />
      
      <div className="flex-1 flex">
        <Sidebar />
        
        <div className="flex-1 relative">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            nodeTypes={nodeTypes}
            fitView
            className="bg-gray-50"
          >
            <Background variant="dots" gap={20} className="bg-gray-50" />
            <Controls />
            <MiniMap className="bg-white" />
          </ReactFlow>
        </div>
        
        <div className="flex">
          {selectedNode && <PropertiesPanel />}
          <YamlPreview />
        </div>
      </div>
    </div>
  )
}

export default App

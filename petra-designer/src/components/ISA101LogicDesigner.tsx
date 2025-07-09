import React, { useState, useCallback, useRef } from 'react';
import { ReactFlow,
  Node,
  Edge,
  Controls,
  Background,
  BackgroundVariant,
  MiniMap,
  NodeTypes,
  addEdge,
  Connection,
  useNodesState,
  useEdgesState,
  ReactFlowProvider
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

// ISA-101 compliant colors
const ISA_COLORS = {
  background: '#D3D3D3',
  node: '#E0E0E0',
  nodeBorder: '#404040',
  nodeSelected: '#000080',
  edge: '#000000',
  text: '#000000',
  running: '#00C800',
  stopped: '#808080',
  alarm: '#FF0000'
};

// Custom Node Component
const LogicBlockNode = ({ data, selected }: { data: any; selected: boolean }) => {
  return (
    <div 
      className={`
        px-4 py-2 rounded-none border-2 
        ${selected ? 'border-blue-800' : 'border-gray-700'}
        ${data.status === 'running' ? 'bg-green-100' : 'bg-gray-200'}
      `}
      style={{
        borderColor: selected ? ISA_COLORS.nodeSelected : ISA_COLORS.nodeBorder,
        backgroundColor: data.status === 'running' ? '#E8F5E8' : ISA_COLORS.node,
        minWidth: '120px'
      }}
    >
      <div className="text-xs font-bold" style={{ color: ISA_COLORS.text }}>
        {data.blockType}
      </div>
      <div className="text-sm" style={{ color: ISA_COLORS.text }}>
        {data.label}
      </div>
      {data.value !== undefined && (
        <div className="text-xs mt-1 font-mono bg-white px-1 border border-gray-600">
          {data.value}
        </div>
      )}
    </div>
  );
};

const nodeTypes: NodeTypes = {
  logicBlock: LogicBlockNode,
};

// Toolbar Component
const ISA101Toolbar = ({ 
  onAddBlock, 
  onDelete, 
  onValidate, 
  onDeploy,
  selectedNode 
}: {
  onAddBlock: (type: string) => void;
  onDelete: () => void;
  onValidate: () => void;
  onDeploy: () => void;
  selectedNode: Node | null;
}) => {
  const blockTypes = [
    { type: 'AND', category: 'Logic' },
    { type: 'OR', category: 'Logic' },
    { type: 'NOT', category: 'Logic' },
    { type: 'GT', category: 'Compare' },
    { type: 'LT', category: 'Compare' },
    { type: 'EQ', category: 'Compare' },
    { type: 'ADD', category: 'Math' },
    { type: 'SUB', category: 'Math' },
    { type: 'MUL', category: 'Math' },
    { type: 'TIMER', category: 'Time' },
    { type: 'PID', category: 'Control' },
  ];

  const categories = ['Logic', 'Compare', 'Math', 'Time', 'Control'];

  return (
    <div className="isa101-toolbar">
      {/* Block Categories */}
      {categories.map(category => (
        <div key={category} className="isa101-toolbar-group">
          <span className="text-xs font-medium mr-2">{category}:</span>
          {blockTypes
            .filter(b => b.category === category)
            .map(block => (
              <button
                key={block.type}
                onClick={() => onAddBlock(block.type)}
                className="isa101-button text-xs px-2 py-1"
                title={`Add ${block.type} block`}
              >
                {block.type}
              </button>
            ))}
        </div>
      ))}
      
      {/* Actions */}
      <div className="isa101-toolbar-group ml-auto">
        <button
          onClick={onDelete}
          disabled={!selectedNode}
          className="isa101-button text-xs px-3 py-1"
          title="Delete selected block"
        >
          Delete
        </button>
        <button
          onClick={onValidate}
          className="isa101-button text-xs px-3 py-1"
          title="Validate logic"
        >
          Validate
        </button>
        <button
          onClick={onDeploy}
          className="isa101-button text-xs px-3 py-1 font-bold"
          style={{ 
            backgroundColor: '#00C800', 
            color: 'white',
            borderColor: '#008000'
          }}
          title="Deploy to PETRA"
        >
          Deploy
        </button>
      </div>
    </div>
  );
};

// Main Logic Designer Component
export default function ISA101LogicDesigner() {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node<any>>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge<any>>([]);
  const [selectedNode, setSelectedNode] = useState<Node<any> | null>(null);
  const [nodeId, setNodeId] = useState(1);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onNodeClick = useCallback((event: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  const onAddBlock = useCallback((blockType: string) => {
    const newNode: Node<any> = {
      id: `node_${nodeId}`,
      type: 'logicBlock',
      position: { x: 100 + (nodeId * 50) % 400, y: 100 + Math.floor(nodeId / 8) * 100 },
      data: { 
        label: `${blockType}_${nodeId}`,
        blockType: blockType,
        status: 'stopped',
        value: blockType === 'TIMER' ? '0.0s' : undefined
      },
    };
    setNodes(nds => nds.concat(newNode));
    setNodeId(nodeId + 1);
  }, [nodeId, setNodes]);

  const onDelete = useCallback(() => {
    if (selectedNode) {
      setNodes(nds => nds.filter(node => node.id !== selectedNode.id));
      setEdges(eds =>
        eds.filter(edge => edge.source !== selectedNode.id && edge.target !== selectedNode.id)
      );
      setSelectedNode(null);
    }
  }, [selectedNode, setNodes, setEdges]);

  const onValidate = useCallback(() => {
    console.log('Validating logic configuration...');
    // Add validation logic here
    alert('Logic validation passed!');
  }, []);

  const onDeploy = useCallback(() => {
    console.log('Deploying to PETRA...');
    const config = {
      blocks: nodes.map(node => ({
        name: node.data.label,
        type: node.data.blockType,
        position: node.position
      })),
      connections: edges.map(edge => ({
        source: edge.source,
        target: edge.target
      }))
    };
    console.log('Deployment config:', config);
    alert('Logic deployed successfully!');
  }, [nodes, edges]);

  return (
    <div className="flex flex-col h-screen isa101-mode">
      <ISA101Toolbar
        onAddBlock={onAddBlock}
        onDelete={onDelete}
        onValidate={onValidate}
        onDeploy={onDeploy}
        selectedNode={selectedNode}
      />
      
      <div className="flex-1" style={{ backgroundColor: ISA_COLORS.background }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          nodeTypes={nodeTypes}
          fitView
          style={{ backgroundColor: ISA_COLORS.background }}
        >
          <Background 
            variant={BackgroundVariant.Dots} 
            gap={20} 
            size={1}
            color="#A0A0A0"
          />
          <Controls 
            style={{
              backgroundColor: ISA_COLORS.node,
              border: `1px solid ${ISA_COLORS.nodeBorder}`,
              borderRadius: 0
            }}
          />
          <MiniMap 
            style={{
              backgroundColor: ISA_COLORS.node,
              border: `1px solid ${ISA_COLORS.nodeBorder}`,
              borderRadius: 0
            }}
            nodeColor={node => node.data.status === 'running' ? ISA_COLORS.running : ISA_COLORS.stopped}
          />
        </ReactFlow>
      </div>
      
      {/* Status Bar */}
      <div 
        className="isa101-toolbar" 
        style={{ 
          borderTop: `1px solid ${ISA_COLORS.nodeBorder}`,
          borderBottom: 'none'
        }}
      >
        <div className="text-xs">
          Nodes: {nodes.length} | Connections: {edges.length} | 
          Selected: {selectedNode ? selectedNode.data.label : 'None'}
        </div>
      </div>
    </div>
  );
}

// Wrapper with ReactFlowProvider
export function ISA101LogicDesignerWrapper() {
  return (
    <ReactFlowProvider>
      <ISA101LogicDesigner />
    </ReactFlowProvider>
  );
}

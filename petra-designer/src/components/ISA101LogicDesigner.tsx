// petra-designer/src/components/ISA101LogicDesigner.tsx
import React, { useState, useCallback } from 'react';
import { 
  ReactFlow,
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
  ReactFlowProvider,
  Handle,
  Position,
  NodeProps
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

// Block configurations matching PETRA backend exactly
const BLOCK_CONFIGS = {
  'AND': { inputs: ['a', 'b'], outputs: ['out'], symbol: '&' },
  'OR': { inputs: ['a', 'b'], outputs: ['out'], symbol: '≥1' },
  'NOT': { inputs: ['in'], outputs: ['out'], symbol: '1' },
  'XOR': { inputs: ['a', 'b'], outputs: ['out'], symbol: '=1' },
  'GT': { inputs: ['a', 'b'], outputs: ['out'], symbol: '>' },
  'LT': { inputs: ['a', 'b'], outputs: ['out'], symbol: '<' },
  'EQ': { inputs: ['a', 'b'], outputs: ['out'], symbol: '=' },
  'ADD': { inputs: ['a', 'b'], outputs: ['out'], symbol: '+' },
  'SUB': { inputs: ['a', 'b'], outputs: ['out'], symbol: '-' },
  'MUL': { inputs: ['a', 'b'], outputs: ['out'], symbol: '×' },
  'DIV': { inputs: ['a', 'b'], outputs: ['out'], symbol: '÷' },
  'ON_DELAY': { inputs: ['in', 'pt'], outputs: ['out', 'et'], symbol: 'TON' },
  'OFF_DELAY': { inputs: ['in', 'pt'], outputs: ['out', 'et'], symbol: 'TOF' },
  'PID': { inputs: ['pv', 'sp'], outputs: ['out'], symbol: 'PID' }
};

// Custom Node Component - Fixed typing
const LogicBlockNode = ({ data, selected }: NodeProps) => {
  const blockType = data?.blockType as string;
  const label = data?.label as string;
  const status = data?.status as string;
  const value = data?.value;
  
  const config = BLOCK_CONFIGS[blockType as keyof typeof BLOCK_CONFIGS];
  
  if (!config) return null;

  return (
    <div 
      className="relative bg-white border-2 min-w-[80px] min-h-[60px] flex flex-col items-center justify-center"
      style={{
        borderColor: selected ? ISA_COLORS.nodeSelected : ISA_COLORS.nodeBorder,
        backgroundColor: status === 'running' ? '#E8F5E8' : ISA_COLORS.node,
      }}
    >
      {/* Block symbol */}
      <div className="text-lg font-bold" style={{ color: ISA_COLORS.text }}>
        {config.symbol}
      </div>
      <div className="text-xs" style={{ color: ISA_COLORS.text }}>
        {label || 'Block'}
      </div>
      {value !== undefined && (
        <div className="text-xs mt-1 font-mono bg-white px-1 border border-gray-600">
          {String(value)}
        </div>
      )}

      {/* Input handles */}
      {config.inputs.map((input, idx) => (
        <Handle
          key={input}
          type="target"
          position={Position.Left}
          id={input}
          style={{
            top: `${25 + idx * 15}px`,
            background: '#FF0000',
            width: '8px',
            height: '8px',
            borderRadius: '0px',
            border: '1px solid #000000'
          }}
        />
      ))}

      {/* Output handles */}
      {config.outputs.map((output, idx) => (
        <Handle
          key={output}
          type="source"
          position={Position.Right}
          id={output}
          style={{
            top: `${25 + idx * 15}px`,
            background: '#0000FF',
            width: '8px',
            height: '8px',
            borderRadius: '0px',
            border: '1px solid #000000'
          }}
        />
      ))}
    </div>
  );
};

const nodeTypes: NodeTypes = {
  logicBlock: LogicBlockNode,
};

// Toolbar Component - Fixed typing
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
    { type: 'ON_DELAY', category: 'Timer' },
    { type: 'PID', category: 'Control' },
  ];

  const categories = ['Logic', 'Compare', 'Math', 'Timer', 'Control'];

  return (
    <div 
      className="flex items-center gap-2 p-2 border-b-2"
      style={{ 
        backgroundColor: ISA_COLORS.node,
        borderColor: ISA_COLORS.nodeBorder 
      }}
    >
      {/* Block Categories */}
      {categories.map(category => (
        <div key={category} className="flex items-center gap-1">
          <span className="text-xs font-medium mr-1">{category}:</span>
          {blockTypes
            .filter(b => b.category === category)
            .map(block => (
              <button
                key={block.type}
                onClick={() => onAddBlock(block.type)}
                className="px-2 py-1 text-xs border border-gray-700 bg-white hover:bg-gray-100"
                title={`Add ${block.type} block`}
              >
                {block.type}
              </button>
            ))}
        </div>
      ))}
      
      {/* Actions */}
      <div className="flex items-center gap-2 ml-auto">
        <button
          onClick={onDelete}
          disabled={!selectedNode}
          className="px-3 py-1 text-xs border border-gray-700 bg-white hover:bg-gray-100 disabled:opacity-50"
          title="Delete selected block"
        >
          Delete
        </button>
        <button
          onClick={onValidate}
          className="px-3 py-1 text-xs border border-gray-700 bg-white hover:bg-gray-100"
          title="Validate logic"
        >
          Validate
        </button>
        <button
          onClick={onDeploy}
          className="px-3 py-1 text-xs font-bold text-white"
          style={{ 
            backgroundColor: ISA_COLORS.running,
            borderColor: '#008000',
            border: '1px solid'
          }}
          title="Deploy to PETRA"
        >
          Deploy
        </button>
      </div>
    </div>
  );
};

// Generate PETRA YAML configuration
const generatePetraYaml = (nodes: Node[], edges: Edge[]): string => {
  const signals: any[] = [];
  const blocks: any[] = [];
  const signalMap = new Map<string, string>();
  let signalCounter = 1;

  // Create signals for connections
  edges.forEach(edge => {
    const signalName = `signal_${signalCounter++}`;
    signalMap.set(`${edge.source}.${edge.sourceHandle}`, signalName);
    signalMap.set(`${edge.target}.${edge.targetHandle}`, signalName);
    
    signals.push({
      name: signalName,
      type: 'bool',
      initial: false
    });
  });

  // Create blocks
  nodes.forEach((node, index) => {
    const blockType = node.data?.blockType as string;
    const label = node.data?.label as string;
    
    const config = BLOCK_CONFIGS[blockType as keyof typeof BLOCK_CONFIGS];
    if (!config) return;

    const inputs: Record<string, string> = {};
    const outputs: Record<string, string> = {};

    // Map inputs
    config.inputs.forEach(inputName => {
      const signalName = signalMap.get(`${node.id}.${inputName}`);
      if (signalName) {
        inputs[inputName] = signalName;
      }
    });

    // Map outputs  
    config.outputs.forEach(outputName => {
      const signalName = signalMap.get(`${node.id}.${outputName}`);
      if (signalName) {
        outputs[outputName] = signalName;
      }
    });

    blocks.push({
      name: label || `${blockType}_${index + 1}`,
      type: blockType,
      inputs,
      outputs
    });
  });

  return `# PETRA Configuration
signals:
${signals.map(s => `  - name: ${s.name}
    type: ${s.type}
    initial: ${s.initial}`).join('\n')}

blocks:
${blocks.map(b => `  - name: ${b.name}
    type: ${b.type}
    inputs:
${Object.entries(b.inputs).map(([k, v]) => `      ${k}: ${v}`).join('\n')}
    outputs:
${Object.entries(b.outputs).map(([k, v]) => `      ${k}: ${v}`).join('\n')}`).join('\n')}

scan_time_ms: 100
`;
};

// Main Logic Designer Component
export default function ISA101LogicDesigner() {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node[]>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge[]>([]);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [nodeId, setNodeId] = useState(1);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onNodeClick = useCallback((event: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  const onAddBlock = useCallback((blockType: string) => {
    const config = BLOCK_CONFIGS[blockType as keyof typeof BLOCK_CONFIGS];
    if (!config) {
      console.error(`Unknown block type: ${blockType}`);
      return;
    }

    const newNode: Node = {
      id: `node_${nodeId}`,
      type: 'logicBlock',
      position: { 
        x: 100 + (nodeId * 50) % 400, 
        y: 100 + Math.floor(nodeId / 8) * 100 
      },
      data: { 
        label: `${blockType}_${nodeId}`,
        blockType: blockType,
        status: 'stopped'
      }
    };

    setNodes(nds => [...nds, newNode]);
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
    const errors: string[] = [];
    
    if (nodes.length === 0) {
      errors.push('No blocks in design');
    }

    // Check for unconnected inputs
    nodes.forEach(node => {
      const blockType = node.data?.blockType as string;
      const label = node.data?.label as string;
      const config = BLOCK_CONFIGS[blockType as keyof typeof BLOCK_CONFIGS];
      if (!config) return;

      config.inputs.forEach(input => {
        const hasConnection = edges.some(edge => 
          edge.target === node.id && edge.targetHandle === input
        );
        if (!hasConnection) {
          errors.push(`${label || node.id}: Input '${input}' not connected`);
        }
      });
    });

    if (errors.length === 0) {
      alert('Logic validation passed!');
    } else {
      alert(`Validation warnings:\n${errors.join('\n')}`);
    }
  }, [nodes, edges]);

  const onDeploy = useCallback(() => {
    const yamlConfig = generatePetraYaml(nodes, edges);
    console.log('Generated PETRA YAML:');
    console.log(yamlConfig);
    
    // Download the config
    const blob = new Blob([yamlConfig], { type: 'text/yaml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'petra-config.yaml';
    a.click();
    URL.revokeObjectURL(url);
    
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
            nodeColor={node => (node.data as any)?.status === 'running' ? ISA_COLORS.running : ISA_COLORS.stopped}
          />
        </ReactFlow>
      </div>
      
      {/* Status Bar */}
      <div 
        className="flex items-center justify-between p-2 text-xs border-t-2"
        style={{ 
          backgroundColor: ISA_COLORS.node,
          borderColor: ISA_COLORS.nodeBorder,
          color: ISA_COLORS.text
        }}
      >
        <div>
          Nodes: {nodes.length} | Connections: {edges.length}
        </div>
        <div>
          Selected: {selectedNode ? String((selectedNode.data as any)?.label || 'Block') : 'None'}
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

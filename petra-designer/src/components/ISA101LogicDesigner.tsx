// petra-designer/src/components/LogicDesigner.tsx
import React, { useState, useCallback } from 'react';
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
  ReactFlowProvider,
  Handle,
  Position,
  NodeProps
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

// Keep the ISA-101 colors simple
const ISA_COLORS = {
  background: '#D3D3D3',
  node: '#E0E0E0',
  nodeBorder: '#404040',
  nodeSelected: '#000080',
  text: '#000000',
  running: '#00C800',
  stopped: '#808080'
};

// Simple block configurations - just what we need for the UI
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

// Simple, fast block node component
const SimpleBlockNode = ({ data, selected }: NodeProps) => {
  const config = BLOCK_CONFIGS[data.blockType as keyof typeof BLOCK_CONFIGS];
  
  if (!config) return null;

  return (
    <div 
      className="relative bg-white border-2 min-w-[80px] min-h-[60px] flex flex-col items-center justify-center"
      style={{
        borderColor: selected ? ISA_COLORS.nodeSelected : ISA_COLORS.nodeBorder,
        backgroundColor: ISA_COLORS.node,
      }}
    >
      {/* Block symbol */}
      <div className="text-lg font-bold">{config.symbol}</div>
      <div className="text-xs">{data.label}</div>

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

// Define node types once
const nodeTypes: NodeTypes = {
  block: SimpleBlockNode,
};

// Simple toolbar
const SimpleToolbar = ({ 
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
    { type: 'XOR', category: 'Logic' },
    { type: 'GT', category: 'Compare' },
    { type: 'LT', category: 'Compare' },
    { type: 'EQ', category: 'Compare' },
    { type: 'ADD', category: 'Math' },
    { type: 'SUB', category: 'Math' },
    { type: 'MUL', category: 'Math' },
    { type: 'ON_DELAY', category: 'Timer' },
    { type: 'PID', category: 'Control' }
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
              >
                {block.type}
              </button>
            ))}
        </div>
      ))}
      
      <div className="flex items-center gap-2 ml-auto">
        <button
          onClick={onDelete}
          disabled={!selectedNode}
          className="px-3 py-1 text-xs border border-gray-700 bg-white hover:bg-gray-100 disabled:opacity-50"
        >
          Delete
        </button>
        <button
          onClick={onValidate}
          className="px-3 py-1 text-xs border border-gray-700 bg-white hover:bg-gray-100"
        >
          Validate
        </button>
        <button
          onClick={onDeploy}
          className="px-3 py-1 text-xs font-bold text-white"
          style={{ backgroundColor: ISA_COLORS.running }}
        >
          Deploy
        </button>
      </div>
    </div>
  );
};

// Generate simple PETRA YAML - fixed to match your backend exactly
const generateSimpleYaml = (nodes: Node[], edges: Edge[]): string => {
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
      type: 'bool', // Keep it simple for now
      initial: false
    });
  });

  // Create blocks
  nodes.forEach((node, index) => {
    const config = BLOCK_CONFIGS[node.data.blockType as keyof typeof BLOCK_CONFIGS];
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
      name: node.data.label || `${node.data.blockType}_${index + 1}`,
      type: node.data.blockType,
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

// Main component - keep it simple and fast
export default function SimpleFixedLogicDesigner() {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
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
    const newNode: Node = {
      id: `node_${nodeId}`,
      type: 'block',
      position: { 
        x: 100 + (nodeId * 50) % 400, 
        y: 100 + Math.floor(nodeId / 8) * 100 
      },
      data: { 
        label: `${blockType}_${nodeId}`,
        blockType: blockType
      }
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
    const errors: string[] = [];
    
    if (nodes.length === 0) {
      errors.push('No blocks in design');
    }

    // Check for unconnected inputs
    nodes.forEach(node => {
      const config = BLOCK_CONFIGS[node.data.blockType as keyof typeof BLOCK_CONFIGS];
      if (!config) return;

      config.inputs.forEach(input => {
        const hasConnection = edges.some(edge => 
          edge.target === node.id && edge.targetHandle === input
        );
        if (!hasConnection) {
          errors.push(`${node.data.label}: Input '${input}' not connected`);
        }
      });
    });

    if (errors.length === 0) {
      alert('Validation passed!');
    } else {
      alert(`Validation warnings:\n${errors.join('\n')}`);
    }
  }, [nodes, edges]);

  const onDeploy = useCallback(() => {
    const yamlConfig = generateSimpleYaml(nodes, edges);
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
    
    alert('Configuration exported!');
  }, [nodes, edges]);

  return (
    <div className="flex flex-col h-screen" style={{ backgroundColor: ISA_COLORS.background }}>
      <SimpleToolbar
        onAddBlock={onAddBlock}
        onDelete={onDelete}
        onValidate={onValidate}
        onDeploy={onDeploy}
        selectedNode={selectedNode}
      />
      
      <div className="flex-1">
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
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>
      
      <div 
        className="flex items-center justify-between p-2 text-xs border-t-2"
        style={{ 
          backgroundColor: ISA_COLORS.node,
          borderColor: ISA_COLORS.nodeBorder
        }}
      >
        <div>Nodes: {nodes.length} | Connections: {edges.length}</div>
        <div>Selected: {selectedNode ? selectedNode.data.label : 'None'}</div>
      </div>
    </div>
  );
}

// Wrapper component
export function SimpleFixedLogicDesignerWrapper() {
  return (
    <ReactFlowProvider>
      <SimpleFixedLogicDesigner />
    </ReactFlowProvider>
  );
}

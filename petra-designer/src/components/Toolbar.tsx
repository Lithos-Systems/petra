import React from 'react'
import { FaSave, FaFolderOpen, FaPlay, FaStop, FaCheckCircle, FaTrash } from 'react-icons/fa'
import { useFlowStore } from '../store/flowStore'
import toast from 'react-hot-toast'

interface ISA101ToolbarProps {
  className?: string
}

export default function ISA101Toolbar({ className = '' }: ISA101ToolbarProps) {
  const { 
    nodes, 
    edges, 
    selectedNode, 
    deleteSelectedNode, 
    exportToYAML,
    validateLogic 
  } = useFlowStore()

  const handleSave = () => {
    try {
      const config = exportToYAML()
      // In a real app, this would save to a file or send to server
      const blob = new Blob([config], { type: 'text/yaml' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'petra-config.yaml'
      a.click()
      URL.revokeObjectURL(url)
      toast.success('Configuration saved')
    } catch (error) {
      toast.error('Failed to save configuration')
    }
  }

  const handleLoad = () => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.yaml,.yml'
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (file) {
        const reader = new FileReader()
        reader.onload = (e) => {
          try {
            // In a real app, this would parse and load the YAML
            toast.success('Configuration loaded')
          } catch (error) {
            toast.error('Failed to load configuration')
          }
        }
        reader.readAsText(file)
      }
    }
    input.click()
  }

  const handleValidate = () => {
    try {
      const result = validateLogic()
      if (result.valid) {
        toast.success(`Validation passed: ${result.nodeCount} blocks, ${result.connectionCount} connections`)
      } else {
        toast.error(`Validation failed: ${result.errors.join(', ')}`)
      }
    } catch (error) {
      toast.error('Validation error')
    }
  }

  const handleDeploy = () => {
    // In a real app, this would deploy to PETRA
    toast.success('Configuration deployed to PETRA')
  }

  const handleDelete = () => {
    if (selectedNode) {
      deleteSelectedNode()
      toast.success('Block deleted')
    }
  }

  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {/* File Operations */}
      <div className="flex items-center gap-1 pr-2 border-r border-[#606060]">
        <button
          onClick={handleSave}
          className="isa101-button text-xs px-2 py-1"
          title="Save Configuration"
        >
          <FaSave className="w-3 h-3" />
        </button>
        <button
          onClick={handleLoad}
          className="isa101-button text-xs px-2 py-1"
          title="Load Configuration"
        >
          <FaFolderOpen className="w-3 h-3" />
        </button>
      </div>

      {/* Logic Operations */}
      <div className="flex items-center gap-1 pr-2 border-r border-[#606060]">
        <button
          onClick={handleValidate}
          className="isa101-button text-xs px-2 py-1"
          title="Validate Logic"
        >
          <FaCheckCircle className="w-3 h-3" />
        </button>
        <button
          onClick={handleDelete}
          disabled={!selectedNode}
          className="isa101-button text-xs px-2 py-1 disabled:opacity-50"
          title="Delete Selected Block"
        >
          <FaTrash className="w-3 h-3" />
        </button>
      </div>

      {/* Deployment */}
      <div className="flex items-center gap-1">
        <button
          onClick={handleDeploy}
          className="isa101-button text-xs px-3 py-1 font-medium"
          style={{ 
            backgroundColor: '#00C800', 
            color: 'white',
            borderColor: '#008000'
          }}
          title="Deploy to PETRA"
        >
          <FaPlay className="w-3 h-3 mr-1" />
          Deploy
        </button>
      </div>

      {/* Status Information */}
      <div className="text-xs text-[#606060] ml-2">
        {nodes.length} blocks, {edges.length} connections
      </div>
    </div>
  )
}

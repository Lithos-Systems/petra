// src/components/Toolbar.tsx
import { FaSave, FaUpload, FaTrash, FaFileImport, FaFileExport } from 'react-icons/fa'
import toast from 'react-hot-toast'
import { useFlowStore } from '@/store/flowStore'

export default function Toolbar() {
  const { clearFlow, nodes, edges, loadFlow } = useFlowStore()

  const handleSave = () => {
    localStorage.setItem('petra-flow', JSON.stringify({ nodes, edges }))
    toast.success('Flow saved to browser storage')
  }

  const handleLoad = () => {
    const saved = localStorage.getItem('petra-flow')
    if (!saved) return toast.error('No saved flow found')

    try {
      const { nodes: n, edges: e } = JSON.parse(saved)
      loadFlow(n, e)
      toast.success('Flow loaded successfully')
    } catch (error) {
      toast.error('Failed to load flow')
    }
  }

  const handleExport = () => {
    const data = JSON.stringify({ nodes, edges }, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'petra-flow.json'
    a.click()
    URL.revokeObjectURL(url)
    toast.success('Flow exported')
  }

  const handleImport = () => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.json'
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (!file) return

      try {
        const text = await file.text()
        const { nodes: n, edges: e } = JSON.parse(text)
        loadFlow(n, e)
        toast.success('Flow imported successfully')
      } catch (error) {
        toast.error('Failed to import flow')
      }
    }
    input.click()
  }

  const handleClear = () => {
    if (confirm('Are you sure you want to clear the canvas?')) {
      clearFlow()
      toast.success('Canvas cleared')
    }
  }

  return (
    <div className="bg-white border-b border-gray-200 px-4 py-2">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-bold text-petra-700">Petra Designer</h1>

        <div className="flex items-center gap-2">
          <button onClick={handleSave} className="btn-primary" title="Save to browser (Ctrl+S)">
            <FaSave className="icon" />
            Save
          </button>

          <button onClick={handleLoad} className="btn-secondary" title="Load from browser">
            <FaUpload className="icon" />
            Load
          </button>

          <div className="border-l border-gray-300 mx-2 h-6" />

          <button onClick={handleExport} className="btn-secondary" title="Export to file">
            <FaFileExport className="icon" />
            Export
          </button>

          <button onClick={handleImport} className="btn-secondary" title="Import from file">
            <FaFileImport className="icon" />
            Import
          </button>

          <div className="border-l border-gray-300 mx-2 h-6" />

          <button onClick={handleClear} className="btn-danger" title="Clear all (Ctrl+Shift+D)">
            <FaTrash className="icon" />
            Clear
          </button>
        </div>
      </div>
    </div>
  )
}

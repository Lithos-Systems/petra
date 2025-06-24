import { FaSave, FaUpload, FaTrash, FaPlay } from 'react-icons/fa'
import { useFlowStore } from '@/store/flowStore'
import toast from 'react-hot-toast'

export default function Toolbar() {
  const { clearFlow, nodes, edges } = useFlowStore()

  const handleSave = () => {
    const data = { nodes, edges }
    localStorage.setItem('petra-flow', JSON.stringify(data))
    toast.success('Flow saved to browser storage')
  }

  const handleLoad = () => {
    const saved = localStorage.getItem('petra-flow')
    if (saved) {
      const { nodes, edges } = JSON.parse(saved)
      useFlowStore.getState().loadFlow(nodes, edges)
      toast.success('Flow loaded successfully')
    } else {
      toast.error('No saved flow found')
    }
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
          <button
            onClick={handleSave}
            className="px-3 py-1.5 bg-petra-500 text-white rounded hover:bg-petra-600 flex items-center gap-2"
          >
            <FaSave className="w-4 h-4" />
            Save
          </button>
          
          <button
            onClick={handleLoad}
            className="px-3 py-1.5 bg-gray-500 text-white rounded hover:bg-gray-600 flex items-center gap-2"
          >
            <FaUpload className="w-4 h-4" />
            Load
          </button>
          
          <button
            onClick={handleClear}
            className="px-3 py-1.5 bg-red-500 text-white rounded hover:bg-red-600 flex items-center gap-2"
          >
            <FaTrash className="w-4 h-4" />
            Clear
          </button>
        </div>
      </div>
    </div>
  )
}

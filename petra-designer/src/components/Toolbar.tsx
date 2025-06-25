import { FaSave, FaUpload, FaTrash } from 'react-icons/fa'   // ⬅️ removed FaPlay
import toast from 'react-hot-toast'
import { useFlowStore } from '@/store/flowStore'

export default function Toolbar() {
  const { clearFlow, nodes, edges } = useFlowStore()

  const handleSave = () => {
    localStorage.setItem('petra-flow', JSON.stringify({ nodes, edges }))
    toast.success('Flow saved to browser storage')
  }

  const handleLoad = () => {
    const saved = localStorage.getItem('petra-flow')
    if (!saved) return toast.error('No saved flow found')

    const { nodes: n, edges: e } = JSON.parse(saved)
    useFlowStore.getState().loadFlow(n, e)
    toast.success('Flow loaded successfully')
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
          <button onClick={handleSave} className="btn-primary">
            <FaSave className="icon" />
            Save
          </button>

          <button onClick={handleLoad} className="btn-secondary">
            <FaUpload className="icon" />
            Load
          </button>

          <button onClick={handleClear} className="btn-danger">
            <FaTrash className="icon" />
            Clear
          </button>
        </div>
      </div>
    </div>
  )
}

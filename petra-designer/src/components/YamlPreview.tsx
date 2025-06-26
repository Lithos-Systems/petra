import { useMemo } from 'react'
import Editor from 'react-simple-code-editor'
// @ts-ignore - if types are missing
import { highlight, languages } from 'prismjs'
import 'prismjs/components/prism-yaml'
import 'prismjs/themes/prism-tomorrow.css'

import { useFlowStore } from '../store/flowStore'
import { generateYaml } from '../utils/yamlGenerator'
import { FaCopy, FaDownload } from 'react-icons/fa'
import toast from 'react-hot-toast'

export default function YamlPreview() {
  const { nodes, edges } = useFlowStore()
  
  const yaml = useMemo(() => {
    return generateYaml(nodes, edges)
  }, [nodes, edges])

  const copyToClipboard = () => {
    navigator.clipboard.writeText(yaml)
    toast.success('Copied to clipboard!')
  }

  const downloadYaml = () => {
    const blob = new Blob([yaml], { type: 'text/yaml' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'petra-config.yaml'  // Changed from .json to .yaml
    a.click()
    URL.revokeObjectURL(url)
    toast.success('Downloaded YAML configuration!')
  }

  return (
    <div className="w-96 bg-gray-900 text-white flex flex-col">
      <div className="p-4 border-b border-gray-700 flex items-center justify-between">
        <h3 className="font-semibold">YAML Preview</h3>
        <div className="flex gap-2">
          <button
            onClick={copyToClipboard}
            className="p-2 hover:bg-gray-700 rounded transition-colors"
            title="Copy to clipboard"
          >
            <FaCopy className="w-4 h-4" />
          </button>
          <button
            onClick={downloadYaml}
            className="p-2 hover:bg-gray-700 rounded transition-colors"
            title="Download YAML"
          >
            <FaDownload className="w-4 h-4" />
          </button>
        </div>
      </div>
      
      <div className="flex-1 overflow-auto">
        <Editor
          value={yaml}
          onValueChange={() => {}} // Read-only
          highlight={(code) => highlight(code, languages.yaml, 'yaml')}
          padding={16}
          style={{
            fontFamily: '"Fira code", "Fira Mono", monospace',
            fontSize: 12,
            minHeight: '100%',
          }}
        />
      </div>
    </div>
  )
}

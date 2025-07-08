import { useState } from 'react'
import { FaTrash, FaLock, FaUnlock, FaEye, FaEyeSlash, FaPlus, FaTimes } from 'react-icons/fa'
import { SketchPicker, ColorResult } from 'react-color'
import type { HMIComponent, SignalBinding, Animation } from '@/types/hmi'
import { useFlowStore } from '@/store/flowStore'

interface HMIPropertiesPanelProps {
  component: HMIComponent
  onUpdate: (updates: Partial<HMIComponent>) => void
  onDelete: () => void
}

export default function HMIPropertiesPanel({
  component,
  onUpdate,
  onDelete,
}: HMIPropertiesPanelProps) {
  const [activeTab, setActiveTab] = useState<'properties' | 'bindings' | 'animations' | 'style'>('properties')
  const [showColorPicker, setShowColorPicker] = useState<string | null>(null)

  // Get available signals from the logic designer
  const { nodes } = useFlowStore()
  const signals = nodes.filter(n => n.type === 'signal').map(n => n.data.label || n.id)

  // Animation helpers
  const addAnimation = () => {
    const newAnimation: Animation = {
      id: Date.now().toString(),
      property: 'rotation',
      trigger: {
        type: 'signal',
        signal: '',
        condition: 'equals',
        value: true
      },
      animation: {
        from: 0,
        to: 360,
        duration: 2000,
        easing: 'linear',
        repeat: true
      }
    }
    onUpdate({ animations: [...component.animations, newAnimation] })
  }

  const updateAnimation = (index: number, updates: Partial<Animation>) => {
    const newAnimations = [...component.animations]
    newAnimations[index] = { ...newAnimations[index], ...updates }
    onUpdate({ animations: newAnimations })
  }

  const removeAnimation = (index: number) => {
    onUpdate({ animations: component.animations.filter((_, i) => i !== index) })
  }

  const handlePropertyChange = (key: string, value: any) => {
    onUpdate({
      properties: { ...component.properties, [key]: value },
    })
  }

  const handleStyleChange = (key: string, value: any) => {
    onUpdate({
      style: { ...component.style, [key]: value },
    })
  }

  const addBinding = () => {
    const newBinding: SignalBinding = { property: 'value', signal: '', transform: '' }
    onUpdate({ bindings: [...component.bindings, newBinding] })
  }

  const updateBinding = (index: number, updates: Partial<SignalBinding>) => {
    const newBindings = [...component.bindings]
    newBindings[index] = { ...newBindings[index], ...updates }
    onUpdate({ bindings: newBindings })
  }

  const removeBinding = (index: number) => {
    onUpdate({ bindings: component.bindings.filter((_, i) => i !== index) })
  }

  const renderAnimationsTab = () => (
    <div className="space-y-3">
      <div className="flex justify-between items-center mb-2">
        <h4 className="text-sm font-medium text-gray-700">Animations</h4>
        <button onClick={addAnimation} className="text-petra-600 hover:text-petra-700 p-1">
          <FaPlus className="w-4 h-4" />
        </button>
      </div>
      {component.animations.map((animation, index) => (
        <div key={animation.id} className="p-3 bg-gray-50 rounded-md space-y-2">
          <div className="flex justify-between items-center">
            <span className="text-sm font-medium">Animation {index + 1}</span>
            <button onClick={() => removeAnimation(index)} className="text-red-500 hover:text-red-600 p-1">
              <FaTimes className="w-3 h-3" />
            </button>
          </div>
          <select
            value={animation.property}
            onChange={(e) => updateAnimation(index, { ...animation, property: e.target.value })}
            className="w-full px-2 py-1 text-sm border border-gray-300 rounded"
          >
            <option value="rotation">Rotation</option>
            <option value="opacity">Opacity</option>
            <option value="scale">Scale</option>
            <option value="x">X Position</option>
            <option value="y">Y Position</option>
          </select>
          <div className="grid grid-cols-2 gap-2">
            <input
              type="number"
              placeholder="From"
              value={animation.animation.from}
              onChange={(e) => updateAnimation(index, { ...animation, animation: { ...animation.animation, from: parseFloat(e.target.value) } })}
              className="px-2 py-1 text-sm border border-gray-300 rounded"
            />
            <input
              type="number"
              placeholder="To"
              value={animation.animation.to}
              onChange={(e) => updateAnimation(index, { ...animation, animation: { ...animation.animation, to: parseFloat(e.target.value) } })}
              className="px-2 py-1 text-sm border border-gray-300 rounded"
            />
          </div>
          <input
            type="number"
            placeholder="Duration (ms)"
            value={animation.animation.duration}
            onChange={(e) => updateAnimation(index, { ...animation, animation: { ...animation.animation, duration: parseInt(e.target.value) } })}
            className="w-full px-2 py-1 text-sm border border-gray-300 rounded"
          />
          <label className="flex items-center text-sm">
            <input
              type="checkbox"
              checked={animation.animation.repeat || false}
              onChange={(e) => updateAnimation(index, { ...animation, animation: { ...animation.animation, repeat: e.target.checked } })}
              className="mr-2"
            />
            Repeat
          </label>
        </div>
      ))}
      {component.animations.length === 0 && (
        <p className="text-sm text-gray-500 text-center py-4">No animations configured. Click + to add one.</p>
      )}
    </div>
  )

  const renderPropertiesTab = () => {
    const propertyFields = getPropertyFields(component.type)
    return (
      <div className="space-y-3">
        {propertyFields.map((field) => (
          <div key={field.key}>
            <label className="block text-sm font-medium text-gray-700 mb-1">{field.label}</label>
            {field.type === 'number' ? (
              <input
                type="number"
                value={component.properties[field.key] || 0}
                onChange={(e) => handlePropertyChange(field.key, parseFloat(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
                step={field.step || 1}
                min={field.min}
                max={field.max}
              />
            ) : field.type === 'boolean' ? (
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={component.properties[field.key] || false}
                  onChange={(e) => handlePropertyChange(field.key, e.target.checked)}
                  className="mr-2"
                />
                <span className="text-sm text-gray-600">{field.label}</span>
              </label>
            ) : field.type === 'select' ? (
              <select
                value={component.properties[field.key] || ''}
                onChange={(e) => handlePropertyChange(field.key, e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                {field.options?.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
            ) : (
              <input
                type="text"
                value={component.properties[field.key] || ''}
                onChange={(e) => handlePropertyChange(field.key, e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            )}
          </div>
        ))}
      </div>
    )
  }

  const renderBindingsTab = () => (
    <div className="space-y-3">
      <div className="flex justify-between items-center mb-2">
        <h4 className="text-sm font-medium text-gray-700">Signal Bindings</h4>
        <button onClick={addBinding} className="text-petra-600 hover:text-petra-700 p-1">
          <FaPlus className="w-4 h-4" />
        </button>
      </div>
      {component.bindings.map((binding, index) => (
        <div key={index} className="p-3 bg-gray-50 rounded-md space-y-2">
          <div className="flex justify-between items-center">
            <span className="text-sm font-medium">Binding {index + 1}</span>
            <button onClick={() => removeBinding(index)} className="text-red-500 hover:text-red-600 p-1">
              <FaTimes className="w-3 h-3" />
            </button>
          </div>
          <select
            value={binding.property}
            onChange={(e) => updateBinding(index, { property: e.target.value })}
            className="w-full px-2 py-1 text-sm border border-gray-300 rounded"
          >
            <option value="">Select property...</option>
            {getBindableProperties(component.type).map((prop) => (
              <option key={prop} value={prop}>{prop}</option>
            ))}
          </select>
          <select
            value={binding.signal}
            onChange={(e) => updateBinding(index, { signal: e.target.value })}
            className="w-full px-2 py-1 text-sm border border-gray-300 rounded"
          >
            <option value="">Select signal...</option>
            {signals.map((signal) => (
              <option key={signal as string} value={signal as string}>{signal as string}</option>
            ))}
          </select>
          <input
            type="text"
            placeholder="Transform (optional JS expression)"
            value={binding.transform || ''}
            onChange={(e) => updateBinding(index, { transform: e.target.value })}
            className="w-full px-2 py-1 text-sm border border-gray-300 rounded"
          />
        </div>
      ))}
      {component.bindings.length === 0 && (
        <p className="text-sm text-gray-500 text-center py-4">No bindings configured. Click + to add one.</p>
      )}
    </div>
  )

  const renderStyleTab = () => (
    <div className="space-y-3">
      {/* Fill Color */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Fill Color</label>
        <div className="relative">
          <button
            onClick={() => setShowColorPicker(showColorPicker === 'fill' ? null : 'fill')}
            className="w-full px-3 py-2 border border-gray-300 rounded-md flex items-center justify-between"
          >
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded border border-gray-300" style={{ backgroundColor: component.style.fill || '#cccccc' }} />
              <span className="text-sm">{component.style.fill || '#cccccc'}</span>
            </div>
          </button>
          {showColorPicker === 'fill' && (
            <div className="absolute z-10 mt-1">
              <SketchPicker color={component.style.fill || '#cccccc'} onChange={(color: ColorResult) => handleStyleChange('fill', color.hex)} />
            </div>
          )}
        </div>
      </div>
      {/* Stroke Color */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Stroke Color</label>
        <div className="relative">
          <button
            onClick={() => setShowColorPicker(showColorPicker === 'stroke' ? null : 'stroke')}
            className="w-full px-3 py-2 border border-gray-300 rounded-md flex items-center justify-between"
          >
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded border border-gray-300" style={{ backgroundColor: component.style.stroke || '#333333' }} />
              <span className="text-sm">{component.style.stroke || '#333333'}</span>
            </div>
          </button>
          {showColorPicker === 'stroke' && (
            <div className="absolute z-10 mt-1">
              <SketchPicker color={component.style.stroke || '#333333'} onChange={(color: ColorResult) => handleStyleChange('stroke', color.hex)} />
            </div>
          )}
        </div>
      </div>
      {/* Stroke Width */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Stroke Width</label>
        <input
          type="number"
          value={component.style.strokeWidth || 1}
          onChange={(e) => handleStyleChange('strokeWidth', parseInt(e.target.value))}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
          min={0}
          max={10}
        />
      </div>
      {/* Opacity */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Opacity</label>
        <input
          type="range"
          value={(component.style.opacity || 1) * 100}
          onChange={(e) => handleStyleChange('opacity', parseInt(e.target.value) / 100)}
          className="w-full"
          min={0}
          max={100}
        />
        <span className="text-sm text-gray-500">{Math.round((component.style.opacity || 1) * 100)}%</span>
      </div>
    </div>
  )

  return (
    <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-lg font-semibold text-gray-800">Properties</h3>
          <div className="flex items-center gap-2">
            <button onClick={() => onUpdate({ locked: !component.locked })} className="p-2 text-gray-600 hover:text-gray-800" title={component.locked ? 'Unlock' : 'Lock'}>
              {component.locked ? <FaLock className="w-4 h-4" /> : <FaUnlock className="w-4 h-4" />}
            </button>
            <button onClick={() => onUpdate({ visible: !component.visible })} className="p-2 text-gray-600 hover:text-gray-800" title={component.visible ? 'Hide' : 'Show'}>
              {component.visible ? <FaEyeSlash className="w-4 h-4" /> : <FaEye className="w-4 h-4" />}
            </button>
            <button onClick={onDelete} className="p-2 text-red-600 hover:text-red-800" title="Delete">
              <FaTrash className="w-4 h-4" />
            </button>
          </div>
        </div>
        <div className="flex space-x-2">
          <button className={`px-3 py-1 rounded-md text-sm ${activeTab === 'properties' ? 'bg-petra-600 text-white' : 'bg-gray-200 text-gray-700'}`} onClick={() => setActiveTab('properties')}>
            Properties
          </button>
          <button className={`px-3 py-1 rounded-md text-sm ${activeTab === 'bindings' ? 'bg-petra-600 text-white' : 'bg-gray-200 text-gray-700'}`} onClick={() => setActiveTab('bindings')}>
            Bindings
          </button>
          <button className={`px-3 py-1 rounded-md text-sm ${activeTab === 'animations' ? 'bg-petra-600 text-white' : 'bg-gray-200 text-gray-700'}`} onClick={() => setActiveTab('animations')}>
            Animations
          </button>
          <button className={`px-3 py-1 rounded-md text-sm ${activeTab === 'style' ? 'bg-petra-600 text-white' : 'bg-gray-200 text-gray-700'}`} onClick={() => setActiveTab('style')}>
            Style
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {activeTab === 'properties' && renderPropertiesTab()}
        {activeTab === 'bindings' && renderBindingsTab()}
        {activeTab === 'animations' && renderAnimationsTab()}
        {activeTab === 'style' && renderStyleTab()}
      </div>
    </div>
  )
}

    ],
    text: [
      { key: 'text', label: 'Text', type: 'text' },
      {
        key: 'align',
        label: 'Alignment',
        type: 'select',
        options: [
          { value: 'left', label: 'Left' },
          { value: 'center', label: 'Center' },
          { value: 'right', label: 'Right' },
        ]
      },
    ],
    'heat-exchanger': [
      { key: 'hotInletTemp', label: 'Hot Inlet Temp (°C)', type: 'number', min: 0, max: 200 },
      { key: 'hotOutletTemp', label: 'Hot Outlet Temp (°C)', type: 'number', min: 0, max: 200 },
      { key: 'coldInletTemp', label: 'Cold Inlet Temp (°C)', type: 'number', min: 0, max: 200 },
      { key: 'coldOutletTemp', label: 'Cold Outlet Temp (°C)', type: 'number', min: 0, max: 200 },
      { key: 'efficiency', label: 'Efficiency (%)', type: 'number', min: 0, max: 100 },
      { key: 'showTemperatures', label: 'Show Temperatures', type: 'boolean' },
    ],
    conveyor: [
      { key: 'running', label: 'Running', type: 'boolean' },
      { key: 'speed', label: 'Speed (%)', type: 'number', min: 0, max: 100 },
      {
        key: 'direction',
        label: 'Direction',
        type: 'select',
        options: [
          { value: 'forward', label: 'Forward' },
          { value: 'reverse', label: 'Reverse' },
        ]
      },
      { key: 'material', label: 'Has Material', type: 'boolean' },
    ],
    mixer: [
      { key: 'running', label: 'Running', type: 'boolean' },
      { key: 'speed', label: 'Speed (RPM)', type: 'number', min: 0, max: 200 },
      { key: 'level', label: 'Level (%)', type: 'number', min: 0, max: 100 },
      {
        key: 'agitatorType',
        label: 'Agitator Type',
        type: 'select',
        options: [
          { value: 'paddle', label: 'Paddle' },
          { value: 'turbine', label: 'Turbine' },
          { value: 'anchor', label: 'Anchor' },
        ]
      },
      { key: 'temperature', label: 'Temperature (°C)', type: 'number', min: -50, max: 200 },
    ],
    motor: [
      { key: 'running', label: 'Running', type: 'boolean' },
      { key: 'speed', label: 'Speed (%)', type: 'number', min: 0, max: 100 },
      { key: 'fault', label: 'Fault', type: 'boolean' },
    ],
    indicator: [
      { key: 'on', label: 'On', type: 'boolean' },
      { key: 'onColor', label: 'On Color', type: 'text' },
      { key: 'offColor', label: 'Off Color', type: 'text' },
    ],
  }

  return fields[type] || []
}

function getBindableProperties(type: string): string[] {
  const properties: Record<string, string[]> = {
    tank: ['currentLevel', 'alarmHigh', 'alarmLow', 'visible'],
    pump: ['running', 'fault', 'speed', 'visible'],
    valve: ['open', 'position', 'fault', 'visible'],
    gauge: ['value', 'min', 'max', 'visible'],
    text: ['text', 'visible'],
    button: ['enabled', 'text', 'visible'],
    indicator: ['on', 'onColor', 'visible'],
  }

  return properties[type] || ['visible']
}

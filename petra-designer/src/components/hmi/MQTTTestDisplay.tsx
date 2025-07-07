// src/components/hmi/MQTTTestDisplay.tsx

import { useState } from 'react'
import { useMQTTTopic, usePetraSignal } from '@/hooks/usePetraConnection'
import { FaWifi, FaWifiSlash } from 'react-icons/fa'

export default function MQTTTestDisplay() {
  const [testTopic, setTestTopic] = useState('sensors/temperature/tank1')
  const [testSignal, setTestSignal] = useState('tank1.level')
  
  // Subscribe to MQTT topic
  const mqttValue = useMQTTTopic(testTopic)
  
  // Subscribe to internal signal
  const signalValue = usePetraSignal(testSignal)
  
  return (
    <div className="fixed bottom-4 right-4 bg-white p-4 rounded-lg shadow-lg border border-gray-200 w-96">
      <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
        <FaWifi className="text-green-500" />
        Real-Time Data Test
      </h3>
      
      {/* MQTT Section */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-1">
          MQTT Topic
        </label>
        <input
          type="text"
          value={testTopic}
          onChange={(e) => setTestTopic(e.target.value)}
          className="w-full px-3 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-petra-500"
          placeholder="sensors/temperature/tank1"
        />
        <div className="mt-2 p-3 bg-gray-50 rounded">
          <div className="text-xs text-gray-500">Topic: {testTopic}</div>
          <div className="text-lg font-mono">
            {mqttValue !== null ? (
              <span className="text-green-600">{JSON.stringify(mqttValue)}</span>
            ) : (
              <span className="text-gray-400">No data</span>
            )}
          </div>
        </div>
      </div>
      
      {/* Signal Section */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-1">
          PETRA Signal
        </label>
        <input
          type="text"
          value={testSignal}
          onChange={(e) => setTestSignal(e.target.value)}
          className="w-full px-3 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-petra-500"
          placeholder="tank1.level"
        />
        <div className="mt-2 p-3 bg-gray-50 rounded">
          <div className="text-xs text-gray-500">Signal: {testSignal}</div>
          <div className="text-lg font-mono">
            {signalValue !== null ? (
              <span className="text-blue-600">{JSON.stringify(signalValue)}</span>
            ) : (
              <span className="text-gray-400">No data</span>
            )}
          </div>
        </div>
      </div>
      
      {/* Example Topics */}
      <div className="text-xs text-gray-500">
        <div className="font-medium mb-1">Example topics:</div>
        <div>• sensors/temperature/+</div>
        <div>• sensors/pressure/+</div>
        <div>• petra/signals/+</div>
        <div>• devices/+/status</div>
      </div>
    </div>
  )
}

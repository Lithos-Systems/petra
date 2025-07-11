// petra-designer/src/main.tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './styles/index.css'
import '@xyflow/react/dist/style.css'

// Remove React.StrictMode for better React Flow performance
ReactDOM.createRoot(document.getElementById('root')!).render(
  <App />
)

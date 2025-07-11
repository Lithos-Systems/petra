// petra-designer/src/main.tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './styles/index.css'
import '@xyflow/react/dist/style.css'

// REMOVED React.StrictMode - this was causing the memory issues
ReactDOM.createRoot(document.getElementById('root')!).render(
  <App />
)

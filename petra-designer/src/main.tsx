import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './styles/index.css'
import '@xyflow/react/dist/style.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  // React Flow performs poorly with StrictMode due to double renders
  // in development. Avoid wrapping the app with React.StrictMode
  <App />,
)

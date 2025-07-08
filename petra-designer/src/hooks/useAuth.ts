// src/hooks/useAuth.ts
import { useState } from 'react'

interface Credentials {
  username: string
  password: string
}

interface User {
  id: string
  name: string
  roles: string[]
  permissions: string[]
}

export function useAuth() {
  const [user, setUser] = useState<User | null>(null)
  const [permissions, setPermissions] = useState<Set<string>>(new Set())

  const login = async (credentials: Credentials) => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(credentials)
    })

    if (response.ok) {
      const { token, user: u } = await response.json()
      localStorage.setItem('petra_token', token)
      setUser(u)
      setPermissions(new Set(u.permissions))
    }
  }

  const logout = () => {
    localStorage.removeItem('petra_token')
    setUser(null)
    setPermissions(new Set())
  }

  const hasPermission = (permission: string) => permissions.has(permission)
  const hasRole = (role: string) => user?.roles?.includes(role) ?? false

  return { user, login, logout, hasPermission, hasRole }
}

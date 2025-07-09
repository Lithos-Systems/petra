import React from 'react'

export function Badge({ children, className }: { children: React.ReactNode; className?: string }) {
  return <span className={className}>{children}</span>
}

import { useContext } from 'react'
import { BackgroundContext } from './context'

/**
 * Hook for using the background context.
 * Separated into its own module to keep component-only exports in index.tsx,
 * which preserves fast-refresh behavior during development.
 */
export function useBackground() {
  const context = useContext(BackgroundContext)
  if (!context) {
    throw new Error('useBackground must be used within a BackgroundProvider')
  }
  return context
}



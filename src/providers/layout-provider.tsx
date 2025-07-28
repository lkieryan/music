import { createContext, useContext } from 'react'

export interface LayoutState {
  rightSide: boolean
  singleToolbar: boolean
  sidebarExpanded: boolean
  isFullscreen: boolean
  compactMode?: boolean
}

const LayoutContext = createContext<LayoutState | null>(null)

export const LayoutProvider = LayoutContext.Provider

// eslint-disable-next-line react-refresh/only-export-components
export function useDesktopLayout(): LayoutState {
  const ctx = useContext(LayoutContext)
  if (!ctx) {
    throw new Error('useDesktopLayout must be used within LayoutProvider')
  }
  return ctx
}


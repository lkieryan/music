import { createContext } from 'react'
import type { GradientState, GradientInternalState } from '~/types/gradient'

export interface BackgroundContextValue {
  // State
  gradientState: GradientState | null
  internalState: GradientInternalState | null

  // Actions
  updateGradient: (gradientState: GradientState, internalState: GradientInternalState) => void
  clearGradient: () => void

  // Flags
  isInitialized: boolean
}

export const BackgroundContext = createContext<BackgroundContextValue | null>(null)



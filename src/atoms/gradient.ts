// import { atom } from 'jotai' // temporarily unused
import { atomWithStorage } from 'jotai/utils'
import type { GradientState, GradientInternalState } from '~/types/gradient'

// Use localStorage to persist gradient output state (for applying to DOM)
export const gradientStateAtom = atomWithStorage<GradientState | null>('gradient-state', null)

// Use localStorage to persist gradient internal state (for editor initialization)
export const gradientInternalStateAtom = atomWithStorage<GradientInternalState | null>('gradient-internal-state', null)
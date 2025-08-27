import type { FC } from 'react'
import { useEffect } from 'react'
import GradientBackground from './gradient'
import DynamicCoverBackground from './dynamic-cover'
import { useBackgroundModeValue } from "~/atoms/settings/themes"
import { useBackground } from '~/providers/background/useBackground'

/**
 * Background (central hub)
 * Choose background variant based on app background mode.
 */
export const Background: FC = () => {
  const mode = useBackgroundModeValue()
  const backgroundContext = useBackground()
  
  // Control window transparency based on background mode
  useEffect(() => {
    if (mode === 'dynamic_cover') {
      // Dynamic cover background: always opaque (disable transparency)
      document.documentElement.setAttribute('data-window-transparent', 'false')
    } else if (mode === 'gradient' && backgroundContext?.isInitialized) {
      // Gradient mode: let gradient logic control transparency
      // Re-trigger transparency calculation based on current gradient state
      const currentInternalState = backgroundContext.internalState || { dots: [], opacity: 1, texture: 0 }
      const hasColorDots = currentInternalState.dots && currentInternalState.dots.length > 0
      
      if (hasColorDots) {
        document.documentElement.setAttribute('data-window-transparent', 'false')
      } else {
        document.documentElement.setAttribute('data-window-transparent', 'true')
      }
    }
  }, [mode, backgroundContext?.isInitialized, backgroundContext?.internalState])
  
  if (mode === 'dynamic_cover') return <DynamicCoverBackground />
  return <GradientBackground />
}
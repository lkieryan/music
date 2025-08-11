import { useCallback, useEffect, useState, useRef, PropsWithChildren } from 'react'
import type { GradientState, GradientInternalState } from '~/types/gradient'
import { useAtomValue } from 'jotai'
import { resolvedThemeAtom } from '~/atoms/settings/themes'
import { GradientGenerator } from '~/lib/gradient-generator'
import { BackgroundContext, type BackgroundContextValue } from './context'

// Context moved to ./context to keep component-only exports for fast-refresh

// Apply gradient values to the DOM (CSS variables and attributes)
const applyGradientToDOM = (data: GradientState) => {
  // Apply the gradient to CSS variables
  // Crossfade: move previous background to *-old and reset opacity for fade
  const previousBg = getComputedStyle(document.documentElement).getPropertyValue('--main-browser-background')
  if (previousBg) {
    document.documentElement.style.setProperty('--main-browser-background-old', previousBg.trim())
  }

  // Set new gradients
  document.documentElement.style.setProperty('--main-browser-background', data.radial)
  document.documentElement.style.setProperty('--main-browser-background-toolbar', data.linear)

  // Opacity transition from previous to 1
  const prevOpacity = getComputedStyle(document.documentElement).getPropertyValue('--background-opacity')
  let startOpacity = 1
  if (prevOpacity) {
    const parsed = parseFloat(prevOpacity)
    if (!Number.isNaN(parsed)) startOpacity = parsed >= 1 ? 0 : 1 - parsed
  }
  document.documentElement.style.setProperty('--background-opacity', `${startOpacity}`)
  requestAnimationFrame(() => {
    document.documentElement.style.setProperty('--background-opacity', `${data.opacity}`)
  })

  // Grain overlay
  document.documentElement.style.setProperty('--grainy-background-opacity', data.texture.toString())
  document.documentElement.setAttribute('show-grainy-background', data.texture > 0 ? 'true' : 'false')

  // Dark mode + toolbox text color
  if (data.toolboxTextColor) {
    document.documentElement.style.setProperty('--toolbox-textcolor', data.toolboxTextColor)
  }
  if (typeof data.shouldBeDark === 'boolean') {
    document.documentElement.setAttribute('should-be-dark-mode', data.shouldBeDark ? 'true' : 'false')
  }
}

// Clear custom gradient values from the DOM
const clearGradientFromDOM = () => {
  // Remove gradient related CSS variables
  document.documentElement.style.removeProperty('--main-browser-background')
  document.documentElement.style.removeProperty('--main-browser-background-toolbar')
  document.documentElement.style.removeProperty('--main-browser-background-old')
  document.documentElement.style.removeProperty('--background-opacity')
  document.documentElement.style.removeProperty('--grainy-background-opacity')
  document.documentElement.removeAttribute('show-grainy-background')
  document.documentElement.removeAttribute('should-be-dark-mode')
  document.documentElement.style.removeProperty('--toolbox-textcolor')
  document.documentElement.style.removeProperty('--base-surface')
}

export function BackgroundProvider({ children }: PropsWithChildren) {
  const [gradientState, setGradientState] = useState<GradientState | null>(null)
  const [internalState, setInternalState] = useState<GradientInternalState | null>(null)
  const [isInitialized, setIsInitialized] = useState(false)
  const resolvedTheme = useAtomValue(resolvedThemeAtom)
  const isDarkMode = resolvedTheme === 'dark'
  
  // Refs to avoid useEffect dependency churn
  const gradientStateRef = useRef<GradientState | null>(null)
  const internalStateRef = useRef<GradientInternalState | null>(null)
  
  // Sync refs with latest state
  gradientStateRef.current = gradientState
  internalStateRef.current = internalState

  // Init: restore gradient state from localStorage
  useEffect(() => {
    try {
      const savedGradientState = localStorage.getItem('gradient-state')
      const savedInternalState = localStorage.getItem('gradient-internal-state')
      
      if (savedGradientState) {
        const gradientData = JSON.parse(savedGradientState)
        setGradientState(gradientData)
        // Apply to DOM immediately
        applyGradientToDOM(gradientData)
      }
      
      if (savedInternalState) {
        const internalData = JSON.parse(savedInternalState)
        setInternalState(internalData)
      }
    } catch (error) {
      console.warn('Failed to restore background state from localStorage:', error)
    } finally {
      setIsInitialized(true)
    }
  }, [])

  // Listen for theme changes and regenerate gradient (same behavior as SchemeButtons)
  useEffect(() => {
    const currentGradientState = gradientStateRef.current
    const currentInternalState = internalStateRef.current
    
    console.log('BackgroundProvider theme change effect triggered:', { 
      isInitialized, 
      hasGradientState: !!currentGradientState, 
      hasInternalState: !!currentInternalState,
      isDarkMode,
      currentShouldBeDark: currentGradientState?.shouldBeDark 
    })
    
    if (!isInitialized) return
    
    if (!currentGradientState || !currentInternalState) {
      console.log('No existing gradient state, creating theme-responsive background')
      
      // Create theme-based background via GradientGenerator
      const themeBasedGradientState = GradientGenerator.generateThemeBasedGradient(isDarkMode)
      
      console.log('Applying theme-based gradient state:', themeBasedGradientState)
      setGradientState(themeBasedGradientState)
      localStorage.setItem('gradient-state', JSON.stringify(themeBasedGradientState))
      applyGradientToDOM(themeBasedGradientState)
      return
    }
    
    // Only regenerate when theme actually changes
    if (currentGradientState.shouldBeDark !== isDarkMode) {
      // Enable a temporary transition gate for UI colors (Plan 1)
      // This lets color-related CSS properties transition smoothly during theme swap.
      const root = document.documentElement
      root.setAttribute('data-theme-transition', 'true')
      const clearTransitionFlag = () => {
        // Ensure the flag exists at least one paint, then remove after duration
        requestAnimationFrame(() => {
          setTimeout(() => {
            root.removeAttribute('data-theme-transition')
          }, 400) // ~350ms duration + small buffer
        })
      }

      console.log('Theme actually changed, regenerating gradient:', {
        from: currentGradientState.shouldBeDark,
        to: isDarkMode
      })
      
      try {
        // Regenerate gradient using GradientGenerator (same logic as SchemeButtons)
        const { dots = [], texture = 0 } = currentInternalState
        if (dots.length > 0) {
          // With color dots: regenerate full gradient
          const updatedGradientState = GradientGenerator.generateFromInternalState(
            currentInternalState,
            isDarkMode,
            isDarkMode, // overrideIsDarkMode
            {
              canBeTransparent: false, // keep original behavior
              isMica: false
            }
          )
          
          console.log('Applying updated gradient state:', updatedGradientState)
          setGradientState(updatedGradientState)
          localStorage.setItem('gradient-state', JSON.stringify(updatedGradientState))
          applyGradientToDOM(updatedGradientState)
          clearTransitionFlag()
        } else {
          const themeBasedGradientState = GradientGenerator.generateThemeBasedGradient(isDarkMode)
          themeBasedGradientState.texture = texture
          themeBasedGradientState.showGrain = texture > 0
          
          console.log('Applying theme-based fallback gradient:', themeBasedGradientState)
          setGradientState(themeBasedGradientState)
          localStorage.setItem('gradient-state', JSON.stringify(themeBasedGradientState))
          applyGradientToDOM(themeBasedGradientState)
          clearTransitionFlag()
        }
      } catch (error) {
        console.warn('Failed to regenerate gradient for theme change:', error)
        clearTransitionFlag()
      }
    }
  }, [isDarkMode, isInitialized])

  const updateGradient = useCallback((newGradientState: GradientState, newInternalState: GradientInternalState) => {
    // Update state
    setGradientState(newGradientState)
    setInternalState(newInternalState)
    
    // Persist to localStorage
    try {
      localStorage.setItem('gradient-state', JSON.stringify(newGradientState))
      localStorage.setItem('gradient-internal-state', JSON.stringify(newInternalState))
    } catch (error) {
      console.warn('Failed to save background state to localStorage:', error)
    }
    
    // Apply to DOM
    applyGradientToDOM(newGradientState)
  }, [])

  const clearGradient = useCallback(() => {
    // Clear state
    setGradientState(null)
    setInternalState(null)
    
    // Clear localStorage
    try {
      localStorage.removeItem('gradient-state')
      localStorage.removeItem('gradient-internal-state')
    } catch (error) {
      console.warn('Failed to clear background state from localStorage:', error)
    }
    
    // Clear DOM
    clearGradientFromDOM()
  }, [])

  const contextValue: BackgroundContextValue = {
    gradientState,
    internalState,
    updateGradient,
    clearGradient,
    isInitialized,
  }

  return (
    <BackgroundContext.Provider value={contextValue}>
      {children}
    </BackgroundContext.Provider>
  )
}

// Hook for using the background context
// Moved to a separate file to avoid fast-refresh complaining about mixed exports

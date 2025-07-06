import { useState, useCallback, useRef, useEffect } from 'react'
import type { ColorDot, GradientState } from '~/types/gradient'
import { MIN_OPACITY, MAX_DOTS, EXPLICIT_LIGHTNESS_TYPE } from '~/components/gradient/constants'
import { getColorFromPosition, calculateCompliments } from '~/lib/color'
import { getGradient, getMostDominantColor, shouldBeDarkMode } from '~/lib/gradient'

export function useGradientState(initialState?: Partial<GradientState>) {
  const [opacity, setOpacity] = useState(initialState?.opacity ?? 0.5)
  const [texture, setTexture] = useState(initialState?.texture ?? 0)
  const [showGrain, setShowGrain] = useState(initialState?.showGrain ?? false)
  // Legacy darken percent (0-50)
  const [legacyDarkenPercent, setLegacyDarkenPercent] = useState<number>(0)
  const [dots, setDots] = useState<ColorDot[]>([])
  const [useAlgo, setUseAlgo] = useState('')
  const [currentLightness, setCurrentLightness] = useState(50)
  const [isDarkMode, setIsDarkMode] = useState(false)
  const [colorPage, setColorPage] = useState(0)
  const [customColors, setCustomColors] = useState<string[]>([])

  // Dragging state
  const [dragging, setDragging] = useState(false)
  const [draggedDot, setDraggedDot] = useState<HTMLElement | null>(null)
  const dragStartPosition = useRef<{ x: number; y: number } | null>(null)
  const recentlyDragged = useRef(false)

  // Platform detection for web
  const canBeTransparent = useRef(false)
  const isMica = useRef(false)

  // Overlay strategy: 'mac' | 'mica' | 'none'
  const [overlayStrategy, setOverlayStrategy] = useState<'mac' | 'mica' | 'none'>('none')

  // Apply overlay strategy by overriding platform flags
  useEffect(() => {
    switch (overlayStrategy) {
      case 'mica':
        isMica.current = true
        canBeTransparent.current = false
        break
      case 'mac':
        isMica.current = false
        canBeTransparent.current = true
        break
      case 'none':
      default:
        isMica.current = false
        canBeTransparent.current = false
        break
    }
  }, [overlayStrategy])

  useEffect(() => {
    // Detect dark mode preference
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    setIsDarkMode(mediaQuery.matches)
    
    const handleChange = (e: MediaQueryListEvent) => setIsDarkMode(e.matches)
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [])

  const createDot = useCallback((color: ColorDot, fromPreset = false) => {
    const [r, g, b] = Array.isArray(color.c) ? color.c : [0, 0, 0]
    
    if (color.isCustom) {
      // Handle custom color dots
      setCustomColors(prev => [...prev, color.c as string])
      return
    }

    const newDot: ColorDot = {
      ID: dots.length,
      c: [r, g, b],
      position: color.position,
      type: color.type,
      lightness: color.lightness,
      isPrimary: dots.length === 0 || color.isPrimary,
    }

    setDots(prevDots => [...prevDots, newDot])
  }, [dots.length])

  const spawnDot = useCallback((position: { x: number; y: number }, primary = false) => {
    const colorFromPos = getColorFromPosition(position.x, position.y, currentLightness)
    const id = primary ? 0 : dots.length

    if (primary && dots.length > 0) {
      // Make existing primary dot non-primary
      setDots(prevDots => prevDots.map(dot => 
        dot.ID === 0 ? { ...dot, ID: dots.length, isPrimary: false } : dot
      ))
    }

    const newDot: ColorDot = {
      ID: id,
      c: colorFromPos,
      position,
      type: EXPLICIT_LIGHTNESS_TYPE,
      lightness: currentLightness,
      isPrimary: primary,
    }

    setDots(prevDots => [...prevDots, newDot])
  }, [dots, currentLightness])

  const handleColorPositions = useCallback((colorPositions: ColorDot[]) => {
    const sortedPositions = [...colorPositions].sort((a, b) => a.ID - b.ID)
    
    const updatedDots = sortedPositions.map(dotPosition => {
      const colorFromPos = getColorFromPosition(
        dotPosition.position!.x,
        dotPosition.position!.y,
        currentLightness,
        dotPosition.type
      )
      
      return {
        ...dotPosition,
        c: colorFromPos,
        lightness: currentLightness,
      }
    })

    setDots(updatedDots)
  }, [currentLightness])

  const addDot = useCallback(() => {
    if (dots.length >= MAX_DOTS) return
    
    const colorPositions = calculateCompliments(dots, 'add', useAlgo)
    handleColorPositions(colorPositions)
  }, [dots, useAlgo, handleColorPositions])

  const removeDot = useCallback(() => {
    if (dots.length === 0) return

    const newDots = [...dots].sort((a, b) => a.ID - b.ID)
    newDots.pop()
    
    // Reassign IDs
    const reassignedDots = newDots.map((dot, index) => ({
      ...dot,
      ID: index,
      isPrimary: index === 0,
    }))

    setDots(reassignedDots)
    
    const colorPositions = calculateCompliments(reassignedDots, 'remove')
    handleColorPositions(colorPositions)
  }, [dots, handleColorPositions])

  const updateDotPosition = useCallback((dotId: number, newPosition: { x: number; y: number }) => {
    setDots(prevDots => {
      const updatedDots = prevDots.map(dot => 
        dot.ID === dotId 
          ? { ...dot, position: newPosition }
          : dot
      )
      
      // Recalculate complimentary positions
      const colorPositions = calculateCompliments(updatedDots, 'update', useAlgo)
      return colorPositions.map(dotPosition => {
        const colorFromPos = getColorFromPosition(
          dotPosition.position!.x,
          dotPosition.position!.y,
          currentLightness,
          dotPosition.type
        )
        
        return {
          ...dotPosition,
          c: colorFromPos,
          lightness: currentLightness,
        }
      })
    })
  }, [useAlgo, currentLightness])

  const addCustomColor = useCallback((color: string, opacity = 1) => {
    let finalColor = color
    
    if (opacity < 1) {
      const hexOpacity = Math.round(opacity * 255)
        .toString(16)
        .padStart(2, '0')
        .toUpperCase()
      
      if (color.startsWith('#') && color.length === 7) {
        finalColor += hexOpacity
      }
    }

    // Add '#' prefix if missing for hex colors
    if (!finalColor.startsWith('#') && /^[0-9A-Fa-f]{3,6}$/.test(finalColor)) {
      finalColor = '#' + finalColor
    }

    setCustomColors(prev => [finalColor, ...prev])
  }, [])

  const removeCustomColor = useCallback((color: string) => {
    setCustomColors(prev => prev.filter(c => c !== color))
  }, [])

  const generateGradient = useCallback((overrideIsDarkMode?: boolean) => {
    const effectiveDark = typeof overrideIsDarkMode === 'boolean' ? overrideIsDarkMode : isDarkMode
    // If there are no color dots, force opacity = 1 (Browser-like behavior)
    const hasColors = dots.length > 0
    const effectiveOpacity = hasColors ? opacity : 1

    // Dark mode resolution
    // In Browser, when legacy mode is enabled, they DO NOT use contrast-based override.
    // They keep window/system dark mode. Mirror that here.
    const legacyEnabled = true
    let resolvedDark = effectiveDark
    if (!legacyEnabled) {
      const dominant = getMostDominantColor(dots)
      resolvedDark = dominant
        ? shouldBeDarkMode(dominant, effectiveDark, canBeTransparent.current, effectiveOpacity)
        : effectiveDark
    }

    console.log('resolvedDark', resolvedDark)

    const radialGradient = getGradient(
      dots,
      effectiveOpacity,
      false,
      resolvedDark,
      true, // legacy darkening
      canBeTransparent.current,
      isMica.current,
      MIN_OPACITY,
      legacyDarkenPercent
    )
    const linearGradient = getGradient(
      dots,
      effectiveOpacity,
      true,
      resolvedDark,
      true, // legacy darkening
      canBeTransparent.current,
      isMica.current,
      MIN_OPACITY,
      legacyDarkenPercent
    )

    // Derive toolbox text color similar to Browser
    const textColor = resolvedDark ? [255, 255, 255, 0.8] : [0, 0, 0, 0.8]
    const toolboxTextColor = `rgba(${textColor[0]}, ${textColor[1]}, ${textColor[2]}, ${textColor[3]})`
    
    const result: GradientState = {
      radial: radialGradient,
      linear: linearGradient,
      opacity: effectiveOpacity,
      texture,
      showGrain: texture > 0,
      shouldBeDark: resolvedDark,
      toolboxTextColor,
    }

    return result
  }, [dots, opacity, texture, isDarkMode])

  const loadPreset = useCallback((
    lightness: number,
    algo: string,
    numDots: number,
    position: string,
    colors?: string[]
  ) => {
    const [x, y] = position.split(',').map(pos => parseInt(pos))
    setCurrentLightness(lightness)
    setUseAlgo(algo)
    
    // Clear existing dots if we have fewer than needed
    if (numDots < dots.length) {
      setDots(prevDots => prevDots.slice(0, numDots))
    }

    let newDots: ColorDot[] = [
      {
        ID: 0,
        position: { x, y },
        isPrimary: true,
        type: EXPLICIT_LIGHTNESS_TYPE,
        c: [0, 0, 0], // Will be calculated
        lightness,
      },
    ]

    for (let i = 1; i < numDots; i++) {
      newDots.push({
        ID: i,
        position: { x: 0, y: 0 },
        type: EXPLICIT_LIGHTNESS_TYPE,
        c: [0, 0, 0], // Will be calculated
        lightness,
      })
    }

    const colorPositions = calculateCompliments(newDots, 'update', algo)
    handleColorPositions(colorPositions)
  }, [dots.length, handleColorPositions])

  return {
    // State
    opacity,
    texture,
    showGrain,
    dots,
    useAlgo,
    currentLightness,
    isDarkMode,
    colorPage,
    customColors,
    dragging,
    draggedDot,
    dragStartPosition,
    recentlyDragged,

    // Actions
    setOpacity,
    setTexture,
    setShowGrain,
    setDots,
    setUseAlgo,
    setCurrentLightness,
    setColorPage,
    createDot,
    spawnDot,
    addDot,
    removeDot,
    updateDotPosition,
    addCustomColor,
    removeCustomColor,
    generateGradient,
    loadPreset,
    setOverlayStrategy,
    setLegacyDarkenPercent,
    
    // Dragging
    setDragging,
    setDraggedDot,
  }
}
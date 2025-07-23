import { useEffect, useCallback, useState } from 'react'
import type { GradientGeneratorDialogProps } from '~/types/gradient'
import { useGradientState } from '~/hooks/common/use-gradient-state'
import GradientPicker from './components/gradient-picker'
import { SaveGradientToThemeButton } from './save-to-theme'
import OpacitySlider from './components/opacity-slider'
import TextureSelector from './components/texture-selector'
import ColorPresets from './components/color-presets'
import ActionButtons from './components/action-buttons'
import SchemeButtons from './components/scheme-buttons'
import { MAX_DOTS } from './constants'
import './index.css'

export default function GradientGeneratorDialog({
  open,
  onClose,
  onChange,
  initialState,
  disabled = false,
}: GradientGeneratorDialogProps) {
  const {
    // State
    opacity,
    texture,
    // showGrain,
    dots,
    // useAlgo,
    currentLightness,
    isDarkMode,
    colorPage,
    customColors,
    dragging,
    draggedDot,
    // dragStartPosition,
    // recentlyDragged,

    // Actions
    setOpacity,
    setTexture,
    // setShowGrain,
    setDots,
    // setUseAlgo,
    // setCurrentLightness,
    setColorPage,
    // createDot,
    spawnDot,
    addDot,
    removeDot,
    updateDotPosition,
    // addCustomColor,
    // removeCustomColor,
    generateGradient,
    
    // Overlay strategy
    // setOverlayStrategy, // declared below under Config section
    loadPreset,
    
    // Config
    setOverlayStrategy,
    setLegacyDarkenPercent,
    
    // Dragging
    setDragging,
    setDraggedDot,
  } = useGradientState(initialState)

  const [scheme, setScheme] = useState<'auto' | 'light' | 'dark'>(isDarkMode ? 'dark' : 'light')

  // Handle gradient changes (Browser-like)
  useEffect(() => {
    if (!onChange) return

    // Use hook's generateGradient as the single source of truth.
    // Only map the scheme override to the boolean passed down.
    const effectiveDark = scheme === 'auto' ? isDarkMode : scheme === 'dark'

    const gradientData = generateGradient(effectiveDark)

    // Mirror Browser: expose should-be-dark-mode and toolbox text color based on resolved theme
    try {
      const shouldDark = gradientData.shouldBeDark ?? effectiveDark
      document.documentElement.setAttribute('should-be-dark-mode', shouldDark ? 'true' : 'false')
      document.documentElement.style.setProperty('--toolbox-textcolor', gradientData.toolboxTextColor || (shouldDark ? 'rgba(255,255,255,0.8)' : 'rgba(0,0,0,0.8)'))
      // optional helper var for any base surface styling
      document.documentElement.style.setProperty('--base-surface', shouldDark ? '#1a1a1a' : '#ffffff')
    } catch {/* no-op */}

    onChange(gradientData)
  }, [opacity, texture, dots, customColors, scheme, isDarkMode, generateGradient, onChange])

  // Handle dot click (create new dot or move primary)
  const handleDotClick = useCallback((position: { x: number; y: number }) => {
    if (dots.length < 1) {
      spawnDot(position, true)
    } else {
      // Move primary dot and recalculate complements
      updateDotPosition(0, position)
    }
  }, [dots.length, spawnDot, updateDotPosition])

  // Handle dot drag
  const handleDotDrag = useCallback((dotId: number, position: { x: number; y: number }) => {
    updateDotPosition(dotId, position)
  }, [updateDotPosition])

  // Handle dot removal (right-click)
  const handleDotRemove = useCallback((dotId: number) => {
    if (dots.length <= 1) return
    
    const filteredDots = dots.filter(dot => dot.ID !== dotId)
    // Reassign IDs
    const reassignedDots = filteredDots.map((dot, index) => ({
      ...dot,
      ID: index,
      isPrimary: index === 0,
    }))
    
    setDots(reassignedDots)
  }, [dots, setDots])

  // Handle preset selection
  const handlePresetSelect = useCallback((
    lightness: number,
    algo: string,
    numDots: number,
    position: string,
    colors?: string[]
  ) => {
    loadPreset(lightness, algo, numDots, position, colors)
  }, [loadPreset])

  // Handle scheme change (simplified for web)
  const handleSchemeChange = useCallback((newScheme: 'auto' | 'light' | 'dark') => {
    // 0 = dark, 1 = light, 2 = auto (match system). Mirror browser mapping.
    // mirror Browser's mapping if needed in future; value unused here

    setScheme(newScheme)

    // Compute effective dark mode based on selected scheme
    const effectiveDark = newScheme === 'auto' ? isDarkMode : (newScheme === 'dark')

    // Re-generate gradients using the override dark flag so color outputs differ subtly
    const next = generateGradient(effectiveDark)
    onChange?.(next)
  }, [isDarkMode, generateGradient, onChange])

  // Handle opacity change with haptic feedback simulation
  const handleOpacityChange = useCallback((newOpacity: number) => {
    setOpacity(newOpacity)
    
    // Simulate haptic feedback for whole number increments
    const rounded = Math.round(newOpacity * 10)
    if (rounded !== Math.round(opacity * 10)) {
      // Could trigger haptic feedback here if available
      console.log('Haptic feedback triggered')
    }
  }, [opacity, setOpacity])

  if (!open) return null

  return (
    <div 
      className="gradient-generator-overlay"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onClose()
        }
      }}
    >
      <div 
        className="gradient-generator-dialog"
        onClick={(e) => e.stopPropagation()}
      >

        <div className="gradient-generator-content">
          {/* Scheme buttons */}
          <SchemeButtons
            currentScheme={scheme}
            onSchemeChange={handleSchemeChange}
          />

          {/* Main gradient picker */}
          <div className="gradient-picker-container">
            <GradientPicker
              dots={dots}
              onDotClick={handleDotClick}
              onDotDrag={handleDotDrag}
              onDotRemove={handleDotRemove}
              dragging={dragging}
              setDragging={setDragging}
              draggedDot={draggedDot}
              setDraggedDot={setDraggedDot}
              currentLightness={currentLightness}
            />

            {/* Action buttons */}
            <ActionButtons
              canAddDot={dots.length < MAX_DOTS}
              canRemoveDot={dots.length > 0}
              onAddDot={addDot}
              onRemoveDot={removeDot}
            />
          </div>

          {/* Color presets */}
          <ColorPresets
            currentPage={colorPage}
            onPageChange={setColorPage}
            onPresetSelect={handlePresetSelect}
          />

          {/* Controls section */}
          <div className="gradient-controls">
            <div className="gradient-controls-wrapper">
              {/* Save to theme */}
              <div style={{ marginBottom: 8 }}>
                <SaveGradientToThemeButton state={generateGradient(isDarkMode)} />
              </div>
              {/* Opacity slider: always rendered; hide only the draggable thumb when no colors */}
              <div className="gradient-opacity-wrapper">
                <OpacitySlider
                  value={opacity}
                  onChange={handleOpacityChange}
                  disabled={disabled}
                  hideThumb={dots.length === 0}
                />
              </div>

              {/* Texture selector */}
              <TextureSelector
                value={texture}
                onChange={setTexture}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
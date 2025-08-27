import { useEffect, useCallback } from 'react'
import type { GradientGeneratorDialogProps, GradientState } from '~/types/gradient'
import { useGradientState } from '~/hooks/common/use-gradient-state'
import { useBackground } from '~/providers/background/useBackground'
import GradientPicker from './components/gradient-picker'
import OpacitySlider from './components/opacity-slider'
import TextureSelector from './components/texture-selector'
import ColorPresets from './components/color-presets'
import ActionButtons from './components/action-buttons'
import SchemeButtons from './components/scheme-buttons'
import { useIsDarkMode, useThemeAtomValue, useSetTheme } from '~/atoms/settings/themes'
import { MAX_DOTS } from '~/constants/gradient'
import './gradient-custom.css'

// 应用渐变到DOM的函数（回退方案）
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

export default function GradientGeneratorDialog({
  open,
  onClose,
  onChange,
  onInternalStateChange,
  initialState,
  disabled = false,
}: GradientGeneratorDialogProps) {
  // 尝试使用BackgroundProvider，如果不可用则回退到旧行为
  let backgroundContext: ReturnType<typeof useBackground> | null = null
  try {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    backgroundContext = useBackground()
  } catch {
    // BackgroundProvider不可用，使用旧行为
    backgroundContext = null
  }
  
  // 使用BackgroundProvider的状态或传入的初始状态
  const effectiveInitialState = backgroundContext?.isInitialized && backgroundContext.internalState 
    ? backgroundContext.internalState 
    : initialState
  
  const {
    // State
    opacity,
    texture,
    // showGrain,
    dots,
    // useAlgo,
    currentLightness,
    // isDarkMode,
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
    // setOverlayStrategy, // 未使用
    // setLegacyDarkenPercent, // 未使用
    
    // Dragging
    setDragging,
    setDraggedDot,
  } = useGradientState(effectiveInitialState)

  // 使用全局的暗色模式状态
  const globalIsDarkMode = useIsDarkMode()
  const currentTheme = useThemeAtomValue()
  const setTheme = useSetTheme()

  // Handle gradient changes
  useEffect(() => {
    // 如果BackgroundProvider可用且未初始化完成，等待
    if (backgroundContext && !backgroundContext.isInitialized) return
    
    // 使用 GradientGenerator 生成渐变数据，保持与原逻辑一致
    const gradientData = generateGradient(globalIsDarkMode)

    const newInternalState = {
      opacity,
      texture,
      showGrain: texture > 0,
      dots,
      currentLightness,
      colorPage,
      customColors,
    }

    // 🔥 关键修复：避免与BackgroundProvider的循环更新
    // 只有当生成的渐变与BackgroundProvider中的状态不同时才更新
    if (backgroundContext) {
      const currentBgState = backgroundContext.gradientState
      const currentInternalState = backgroundContext.internalState
      
      // 检查是否需要更新（避免无意义的重复更新）
      const shouldUpdate = !currentBgState || 
        currentBgState.radial !== gradientData.radial ||
        currentBgState.linear !== gradientData.linear ||
        currentBgState.opacity !== gradientData.opacity ||
        currentBgState.texture !== gradientData.texture ||
        currentBgState.shouldBeDark !== gradientData.shouldBeDark ||
        !currentInternalState ||
        JSON.stringify(currentInternalState.dots) !== JSON.stringify(newInternalState.dots)
      
      if (shouldUpdate) {
        backgroundContext.updateGradient(gradientData, newInternalState)
      }
    }

    // 如果有外部回调，也调用它们（向后兼容或BackgroundProvider不可用时的回退）
    if (onChange) {
      onChange(gradientData)
    }
    if (onInternalStateChange) {
      onInternalStateChange(newInternalState)
    }
    
    // 如果BackgroundProvider不可用，直接应用到DOM（回退方案）
    if (!backgroundContext) {
      applyGradientToDOM(gradientData)
    }
  }, [opacity, texture, dots, customColors, currentLightness, colorPage, generateGradient, backgroundContext, onChange, onInternalStateChange, globalIsDarkMode])

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
    // colors?: string[] // 目前未使用
  ) => {
    loadPreset(lightness, algo, numDots, position)
  }, [loadPreset])

  // Handle scheme change - 调用通用的主题设置
  const handleSchemeChange = useCallback(async (newScheme: 'auto' | 'light' | 'dark') => {
    const themeMap = {
      'auto': 'system',
      'light': 'light', 
      'dark': 'dark'
    } as const
    
    await setTheme(themeMap[newScheme])
  }, [setTheme])

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
      className="fixed inset-0 flex items-center justify-center z-[9999] animate-in fade-in duration-200"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onClose()
        }
      }}
    >
      <div 
        className="bg-white/70 dark:bg-zinc-800/80  rounded-xl shadow-2xl w-[360px] max-h-[90vh] overflow-hidden animate-in zoom-in-95 slide-in-from-bottom-2 duration-300"
        onClick={(e) => e.stopPropagation()}
      >

        <div className="p-2.5 flex flex-col gap-4 relative">
          {/* Scheme buttons - 方便用户调试渐变色 */}
          <SchemeButtons
            currentScheme={currentTheme === 'system' ? 'auto' : currentTheme}
            onSchemeChange={handleSchemeChange}
          />

          {/* Main gradient picker */}
          <div className="relative">
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
          <div className="items-center gap-2.5 pt-2.5">
            <div className="flex justify-between w-full mb-2.5 items-center gap-6 px-2.5">
              {/* Opacity slider: always rendered; hide only the draggable thumb when no colors */}
              <div className="relative flex-1">
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
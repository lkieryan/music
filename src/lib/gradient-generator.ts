import type { ColorDot, GradientState, GradientInternalState } from '~/types/gradient'
import { MIN_OPACITY } from '~/constants/gradient'
import { getGradient, getMostDominantColor, shouldBeDarkMode } from './gradient'

/**
 * Gradient generator configuration
 */
export interface GradientGeneratorConfig {
  /** Color dots */
  dots: ColorDot[]
  /** Opacity (0-1) */
  opacity: number
  /** Texture intensity (0-1) */
  texture: number
  /** Whether dark mode is active */
  isDarkMode: boolean
  /** Legacy darken percent (0-50) */
  legacyDarkenPercent: number
  /** Whether background can be transparent */
  canBeTransparent: boolean
  /** Whether Mica mode is enabled */
  isMica: boolean
}

/**
 * Gradient generator
 * Provides standalone gradient generation that does not depend on React hooks
 */
export class GradientGenerator {
  /**
   * Generate gradient state from configuration
   * Core logic extracted from useGradientState.generateGradient
   */
  static generate(config: GradientGeneratorConfig, overrideIsDarkMode?: boolean): GradientState {
    const {
      dots,
      opacity,
      texture,
      isDarkMode,
      legacyDarkenPercent,
      canBeTransparent,
      isMica
    } = config

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
        ? shouldBeDarkMode(dominant, effectiveDark, canBeTransparent, effectiveOpacity)
        : effectiveDark
    }

    const radialGradient = getGradient(
      dots,
      effectiveOpacity,
      false,
      resolvedDark,
      true, // legacy darkening
      canBeTransparent,
      isMica,
      MIN_OPACITY,
      legacyDarkenPercent
    )
    const linearGradient = getGradient(
      dots,
      effectiveOpacity,
      true,
      resolvedDark,
      true, // legacy darkening
      canBeTransparent,
      isMica,
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
  }

  /**
   * Generate from internal state
   * Convenience helper that builds from GradientInternalState
   */
  static generateFromInternalState(
    internalState: GradientInternalState,
    isDarkMode: boolean,
    overrideIsDarkMode?: boolean,
    platformConfig: {
      canBeTransparent?: boolean
      isMica?: boolean
    } = {}
  ): GradientState {
    const config: GradientGeneratorConfig = {
      dots: internalState.dots ?? [],
      opacity: internalState.opacity ?? 0.5,
      texture: internalState.texture ?? 0,
      isDarkMode,
      legacyDarkenPercent: 0, 
      canBeTransparent: platformConfig.canBeTransparent ?? false,
      isMica: platformConfig.isMica ?? false,
    }

    return this.generate(config, overrideIsDarkMode)
  }

  /**
   * Generate theme-based default gradient
   * Used when there is no custom gradient; matches getGradient() behavior for zero dots
   */
  static generateThemeBasedGradient(isDarkMode: boolean): GradientState {
    // Same logic as getGradient() for zero dots: return simple background color with transparency
    const radialBackground = isDarkMode ? 'rgba(42, 42, 42, 0.6)' : 'rgba(248, 250, 252, 0.8)'
    const linearBackground = isDarkMode ? 'rgba(42, 42, 42, 0.6)' : 'rgba(248, 250, 252, 0.8)'
    
    return {
      radial: radialBackground,
      linear: linearBackground,
      opacity: 1,
      texture: 0,
      showGrain: false,
      shouldBeDark: isDarkMode,
      toolboxTextColor: isDarkMode ? 'rgba(255,255,255,0.8)' : 'rgba(0,0,0,0.8)'
    }
  }
}

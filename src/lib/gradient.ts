import type { ColorDot } from '../types/gradient'
import { blendColors, contrastRatio } from './color'

export function getGradient(
  colors: ColorDot[],
  opacity: number,
  forToolbar = false,
  isDarkMode = false,
  isLegacyVersion = false,
  canBeTransparent = false,
  isMica = false,
  MIN_OPACITY = 0.35,
  legacyDarkenPercent = 30
): string {
  const themedColors = [...colors]
  const rotation = -45

  if (themedColors.length === 0) {
    return forToolbar
      ? getToolbarModifiedBase(isDarkMode)
      : isDarkMode
        ? 'rgba(0, 0, 0, 0.4)'
        : 'transparent'
  } else if (themedColors.length === 1) {
    return getSingleRGBColor(themedColors[0], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY)
  } else {
    // If there are custom colors, return linear gradient with all colors
    if (themedColors.find((color) => color.isCustom)) {
      const gradientColors = themedColors.map((color) =>
        getSingleRGBColor(color, opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)
      )
      const colorStops = gradientColors
        .map((color, index) => {
          const position = (index / (gradientColors.length - 1)) * 100
          return `${color} ${position}%`
        })
        .join(', ')
      return `linear-gradient(${rotation}deg, ${colorStops})`
    }
    
    if (themedColors.length === 2) {
      if (!forToolbar) {
        return [
          `linear-gradient(${rotation}deg, ${getSingleRGBColor(themedColors[1], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)} 0%, transparent 100%)`,
          `linear-gradient(${rotation + 180}deg, ${getSingleRGBColor(themedColors[0], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)} 0%, transparent 80%)`,
        ]
          .reverse()
          .join(', ')
      }
      return `linear-gradient(${rotation}deg, ${getSingleRGBColor(themedColors[1], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)} 0%, ${getSingleRGBColor(themedColors[0], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)} 100%)`
    } else if (themedColors.length === 3) {
      const color1 = getSingleRGBColor(themedColors[2], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)
      const color2 = getSingleRGBColor(themedColors[0], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)
      const color3 = getSingleRGBColor(themedColors[1], opacity, forToolbar, isDarkMode, isLegacyVersion, canBeTransparent, isMica, MIN_OPACITY, legacyDarkenPercent)
      if (!forToolbar) {
        return [
          `radial-gradient(circle at 0% 0%, ${color2}, transparent 100%)`,
          `radial-gradient(circle at 100% 0%, ${color3}, transparent 100%)`,
          `linear-gradient(to top, ${color1} 0%, transparent 60%)`,
        ].join(', ')
      }
      return [`linear-gradient(-45deg, ${color1} 15%, ${color2})`].join(', ')
    }
  }
  
  return 'transparent'
}

function getSingleRGBColor(
  color: ColorDot,
  opacity: number,
  forToolbar = false,
  isDarkMode = false,
  isLegacyVersion = false,
  canBeTransparent = false,
  isMica = false,
  MIN_OPACITY = 0.35,
  legacyDarkenPercent = 60
): string {
  if (color.isCustom) {
    return color.c as string
  }
  
  let rgbColor = color.c as [number, number, number]
  let finalOpacity = opacity
  
  if (forToolbar) {
    rgbColor = blendColors(
      rgbColor,
      getToolbarModifiedBaseRaw(isDarkMode).slice(0, 3) as [number, number, number],
      opacity * 100
    )
    finalOpacity = 1 // Toolbar colors should always be fully opaque
  }
  
  if (isLegacyVersion && isDarkMode) {
    // In legacy version, blend with black overlay in dark mode (extra darkening)
    console.log('legacyDarkenPercent', legacyDarkenPercent)
    rgbColor = blendColors(rgbColor, [0, 0, 0], 60) // TODO: use legacyDarkenPercent
  }
  
  return blendWithWhiteOverlay(rgbColor, finalOpacity, isDarkMode, canBeTransparent, isMica, MIN_OPACITY)
}

function getToolbarModifiedBaseRaw(isDarkMode: boolean): [number, number, number, number] {
  const opacity = 1 // Simplified for web
  return isDarkMode ? [23, 23, 26, opacity] : [240, 240, 244, opacity]
}

function getToolbarModifiedBase(isDarkMode: boolean): string {
  const baseColor = getToolbarModifiedBaseRaw(isDarkMode)
  return `rgba(${baseColor[0]}, ${baseColor[1]}, ${baseColor[2]}, ${baseColor[3]})`
}

function blendWithWhiteOverlay(
  baseColor: [number, number, number],
  opacity: number,
  isDarkMode: boolean,
  canBeTransparent: boolean,
  isMica: boolean,
  MIN_OPACITY: number
): string {
  let colorToBlend: [number, number, number] | undefined
  let colorToBlendOpacity: number = 0
  
  if (isMica) {
    colorToBlend = !isDarkMode ? [0, 0, 0] : [255, 255, 255]
    colorToBlendOpacity = 0.35
  } else if (canBeTransparent) { // Assuming macOS-like behavior for web
    colorToBlend = [255, 255, 255]
    colorToBlendOpacity = 0.3
  }
  
  if (colorToBlend) {
    const blendedAlpha = Math.min(
      1,
      opacity + MIN_OPACITY + colorToBlendOpacity * (1 - (opacity + MIN_OPACITY))
    )
    baseColor = blendColors(baseColor, colorToBlend, blendedAlpha * 100)
    opacity += colorToBlendOpacity * (1 - opacity)
  }
  
  return `rgba(${baseColor[0]}, ${baseColor[1]}, ${baseColor[2]}, ${opacity})`
}

export function shouldBeDarkMode(
  accentColor: [number, number, number],
  isDarkMode: boolean,
  canBeTransparent: boolean,
  currentOpacity: number,
  darkModeBias = 0.25
): boolean {
  if (!canBeTransparent) {
    const toolbarBg = getToolbarModifiedBaseRaw(isDarkMode)
    accentColor = blendColors(
      toolbarBg.slice(0, 3) as [number, number, number],
      accentColor,
      (1 - currentOpacity) * 100
    )
  }

  const bg = accentColor

  // Get text colors (with alpha)
  const darkText: [number, number, number, number] = getToolbarColor(true)
  const lightText: [number, number, number, number] = getToolbarColor(false)

  if (canBeTransparent) {
    lightText[3] -= darkModeBias // Reduce alpha for light text
  }

  // Composite text color over background
  const darkTextRgb = blendColors(bg, darkText.slice(0, 3) as [number, number, number], (1 - darkText[3]) * 100)
  const lightTextRgb = blendColors(bg, lightText.slice(0, 3) as [number, number, number], (1 - lightText[3]) * 100)

  const darkContrast = contrastRatio(bg, darkTextRgb)
  const lightContrast = contrastRatio(bg, lightTextRgb)

  return darkContrast > lightContrast
}

function getToolbarColor(isDarkMode = false): [number, number, number, number] {
  return isDarkMode ? [255, 255, 255, 0.8] : [0, 0, 0, 0.8]
}

export function getMostDominantColor(allColors: ColorDot[]): [number, number, number] | undefined {
  const color = getPrimaryColor(allColors)
  if (typeof color === 'string' || typeof color === 'undefined') return undefined
  if (Array.isArray(color)) {
    const [r, g, b] = color
    return [r, g, b]
  }
  return undefined
}

function getPrimaryColor(colors: ColorDot[]): [number, number, number] | string | undefined {
  const primaryColor = colors.find((color) => color.isPrimary)
  if (primaryColor) {
    const c = primaryColor.c
    if (typeof c === 'string') return c
    if (Array.isArray(c)) {
      const [r, g, b] = c
      return [r, g, b]
    }
    return undefined
  }
  if (colors.length === 0) {
    return undefined
  }
  // Get the middle color
  const mid = colors[Math.floor(colors.length / 2)].c
  if (typeof mid === 'string') return mid
  if (Array.isArray(mid)) {
    const [r, g, b] = mid
    return [r, g, b]
  }
  return undefined
}
import { COLOR_HARMONIES, PICKER_SIZE, PICKER_PADDING } from '../components/gradient/constants'
import type { ColorHarmony, ColorDot } from '../types/gradient'

/**
 * Converts an HSL color value to RGB. Conversion formula
 * adapted from https://en.wikipedia.org/wiki/HSL_color_space.
 * Assumes h, s, and l are contained in the set [0, 1] and
 * returns r, g, and b in the set [0, 255].
 */
export function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const { round } = Math
  let r: number, g: number, b: number

  if (s === 0) {
    r = g = b = l // achromatic
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s
    const p = 2 * l - q
    r = hueToRgb(p, q, h + 1 / 3)
    g = hueToRgb(p, q, h)
    b = hueToRgb(p, q, h - 1 / 3)
  }

  return [round(r * 255), round(g * 255), round(b * 255)]
}

export function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255
  g /= 255
  b /= 255
  const max = Math.max(r, g, b)
  const min = Math.min(r, g, b)
  const d = max - min
  let h: number
  if (d === 0) h = 0
  else if (max === r) h = ((g - b) / d) % 6
  else if (max === g) h = (b - r) / d + 2
  else if (max === b) h = (r - g) / d + 4
  else h = 0
  const l = (min + max) / 2
  const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1))
  return [h * 60, s, l]
}

function hueToRgb(p: number, q: number, t: number): number {
  if (t < 0) t += 1
  if (t > 1) t -= 1
  if (t < 1 / 6) return p + (q - p) * 6 * t
  if (t < 1 / 2) return q
  if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6
  return p
}

export function calculateInitialPosition([r, g, b]: [number, number, number]): { x: number; y: number } {
  const padding = 30
  const rect = {
    width: PICKER_SIZE,
    height: PICKER_SIZE,
  }
  const centerX = rect.width / 2
  const centerY = rect.height / 2
  const radius = (rect.width - padding) / 2
  const [hue, saturation] = rgbToHsl(r, g, b)
  const angle = (hue / 360) * 2 * Math.PI // Convert to radians
  const normalizedSaturation = saturation / 100 // Convert to [0, 1]
  const x = centerX + radius * normalizedSaturation * Math.cos(angle) - padding
  const y = centerY + radius * normalizedSaturation * Math.sin(angle) - padding
  return { x, y }
}

export function getColorFromPosition(
  x: number,
  y: number,
  currentLightness: number,
  type?: string
): [number, number, number] {
  const padding = 30
  const dotHalfSize = 36 / 2
  x += dotHalfSize
  y += dotHalfSize
  
  let rectWidth = PICKER_SIZE + padding * 2
  let rectHeight = PICKER_SIZE + padding * 2
  
  const centerX = rectWidth / 2
  const centerY = rectHeight / 2
  const radius = (rectWidth - padding) / 2
  const distance = Math.sqrt((x - centerX) ** 2 + (y - centerY) ** 2)
  let angle = Math.atan2(y - centerY, x - centerX)
  angle = (angle * 180) / Math.PI
  if (angle < 0) {
    angle += 360
  }
  const normalizedDistance = 1 - Math.min(distance / radius, 1)
  const hue = (angle / 360) * 360
  let saturation = normalizedDistance * 100
  let lightness = currentLightness
  
  if (type !== 'explicit-lightness') {
    saturation = 80 + (1 - normalizedDistance) * 20
    lightness = Math.round((1 - normalizedDistance) * 100)
  }
  
  const [r, g, b] = hslToRgb(hue / 360, saturation / 100, lightness / 100)
  return [
    Math.min(255, Math.max(0, r)),
    Math.min(255, Math.max(0, g)),
    Math.min(255, Math.max(0, b)),
  ]
}

export function luminance([r, g, b]: [number, number, number]): number {
  const a = [r, g, b].map((v) => {
    v /= 255
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
  })
  return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722
}

export function contrastRatio(rgb1: [number, number, number], rgb2: [number, number, number]): number {
  const lum1 = luminance(rgb1)
  const lum2 = luminance(rgb2)
  const brightest = Math.max(lum1, lum2)
  const darkest = Math.min(lum1, lum2)
  return (brightest + 0.05) / (darkest + 0.05)
}

export function blendColors(
  rgb1: [number, number, number],
  rgb2: [number, number, number],
  percentage: number
): [number, number, number] {
  const p = percentage / 100
  return [
    Math.round(rgb1[0] * p + rgb2[0] * (1 - p)),
    Math.round(rgb1[1] * p + rgb2[1] * (1 - p)),
    Math.round(rgb1[2] * p + rgb2[2] * (1 - p)),
  ]
}

export function hexToRgb(hex: string): [number, number, number] {
  if (hex.startsWith('#')) {
    hex = hex.substring(1)
  }
  if (hex.length === 3) {
    hex = hex
      .split('')
      .map((char) => char + char)
      .join('')
  }
  return [
    parseInt(hex.substring(0, 2), 16),
    parseInt(hex.substring(2, 4), 16),
    parseInt(hex.substring(4, 6), 16),
  ]
}

export function calculateCompliments(
  dots: ColorDot[],
  action: 'update' | 'add' | 'remove' = 'update',
  useHarmony = '',
  pickerSize: { width: number; height: number } = { width: PICKER_SIZE, height: PICKER_SIZE }
): ColorDot[] {
  if (dots.length === 0) {
    return []
  }

  function getColorHarmonyType(numDots: number, dots: ColorDot[]): ColorHarmony | undefined {
    if (useHarmony !== '') {
      const selectedHarmony = COLOR_HARMONIES.find((harmony) => harmony.type === useHarmony)

      if (selectedHarmony) {
        if (action === 'remove') {
          if (dots.length !== 0) {
            return COLOR_HARMONIES.find(
              (harmony) => harmony.angles.length === selectedHarmony.angles.length - 1
            )
          } else {
            return { type: 'floating', angles: [] }
          }
        }
        if (action === 'add') {
          return COLOR_HARMONIES.find(
            (harmony) => harmony.angles.length === selectedHarmony.angles.length + 1
          )
        }
        if (action === 'update') {
          return selectedHarmony
        }
      }
    }

    if (action === 'remove') {
      return COLOR_HARMONIES.find((harmony) => harmony.angles.length === numDots)
    }
    if (action === 'add') {
      return COLOR_HARMONIES.find((harmony) => harmony.angles.length + 1 === numDots)
    }
    if (action === 'update') {
      return COLOR_HARMONIES.find((harmony) => harmony.angles.length + 1 === numDots)
    }
  }

  function getAngleFromPosition(position: { x: number; y: number }, centerPosition: { x: number; y: number }): number {
    const deltaX = position.x - centerPosition.x
    const deltaY = position.y - centerPosition.y
    let angle = Math.atan2(deltaY, deltaX) * (180 / Math.PI)
    return (angle + 360) % 360
  }

  function getDistanceFromCenter(position: { x: number; y: number }, centerPosition: { x: number; y: number }): number {
    const deltaX = position.x - centerPosition.x
    const deltaY = position.y - centerPosition.y
    return Math.sqrt(deltaX * deltaX + deltaY * deltaY)
  }

  const padding = PICKER_PADDING
  let updatedDots = [...dots]
  const centerPosition = { x: pickerSize.width / 2, y: pickerSize.height / 2 }

  const harmonyAngles = getColorHarmonyType(
    dots.length + (action === 'add' ? 1 : action === 'remove' ? -1 : 0),
    dots
  )
  
  if (!harmonyAngles || harmonyAngles.angles.length === 0) return dots

  const primaryDot = dots.find((dot) => dot.ID === 0)
  if (!primaryDot || !primaryDot.position) return []

  if (action === 'add' && dots.length) {
    updatedDots.push({ ID: dots.length, position: centerPosition, c: [0, 0, 0] })
  }
  
  const baseAngle = getAngleFromPosition(primaryDot.position, centerPosition)
  let distance = getDistanceFromCenter(primaryDot.position, centerPosition)
  const radius = (pickerSize.width - padding) / 2
  if (distance > radius) distance = radius
  
  if (dots.length > 0) {
    updatedDots = [
      {
        ID: 0,
        position: primaryDot.position,
        type: primaryDot.type,
        c: primaryDot.c,
      },
    ]
  }

  harmonyAngles.angles.forEach((angleOffset, index) => {
    const newAngle = (baseAngle + angleOffset) % 360
    const radian = (newAngle * Math.PI) / 180

    const newPosition = {
      x: centerPosition.x + distance * Math.cos(radian),
      y: centerPosition.y + distance * Math.sin(radian),
    }

    updatedDots.push({
      ID: index + 1,
      position: newPosition,
      type: primaryDot.type,
      c: [0, 0, 0], // Will be calculated later
    })
  })

  return updatedDots
}
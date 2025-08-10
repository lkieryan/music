// import { atom } from 'jotai' // 暂时未使用
import { atomWithStorage } from 'jotai/utils'
import type { GradientState, GradientInternalState } from '~/types/gradient'

// 使用 localStorage 持久化渐变输出状态（用于应用到DOM）
export const gradientStateAtom = atomWithStorage<GradientState | null>('gradient-state', null)

// 使用 localStorage 持久化渐变内部状态（用于编辑器初始化）
export const gradientInternalStateAtom = atomWithStorage<GradientInternalState | null>('gradient-internal-state', null)

// 应用渐变到DOM的函数
export const applyGradientToDOM = (data: GradientState) => {
  console.log("applyGradientToDOM", data)
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

// 清除自定义渐变状态
export const clearCustomGradient = () => {
  // 清除DOM中的渐变样式
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
import { atom, useAtom, useAtomValue } from 'jotai'
import { useEffect } from 'react'
import { useGeneralSettingKey, setGeneral, listenGeneral } from './general'
import { setUISetting } from "~/atoms/settings/ui"
import { jotaiStore } from '~/lib/jotai'
import { createSettingAtom } from "~/atoms/helper/setting"

export type AppTheme = 'light' | 'dark' | 'system'
export type AppBackgroundMode = 'gradient' | 'dynamic_cover'

// 从设置中获取主题值的atom（与后端同步）
export const appThemeAtom = atom<AppTheme>('system')

// 计算实际应用的主题（当选择system时，根据系统主题决定）
export const resolvedThemeAtom = atom<'light' | 'dark'>((get) => {
  const theme = get(appThemeAtom)
  if (theme !== 'system') return theme
  
  // 检测系统主题
  if (typeof window !== 'undefined') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return 'light'
})

// ==================================================================
// Background settings storage under themes domain
// ==================================================================

export type BackgroundRendererKind = 'mesh' | 'css-bg'

export type BackgroundSettingsShape = {
  backgroundRenderer: BackgroundRendererKind
  cssBackgroundProperty: string
  backgroundFps: number
  backgroundRenderScale: number
  backgroundStaticMode: boolean
}

export const createDefaultBackgroundSettings = (): BackgroundSettingsShape => ({
  backgroundRenderer: 'mesh',
  cssBackgroundProperty: '#111111',
  backgroundFps: 60,
  backgroundRenderScale: 1,
  backgroundStaticMode: false,
})

const backgroundSetting = createSettingAtom("background", createDefaultBackgroundSettings)

// export accessors for background settings (themes domain)
export const useBackgroundSettingsValue = backgroundSetting.useSettingValue
export const setBackgroundSetting = backgroundSetting.setSetting

// 检查是否为暗色模式（供渐变生成器等组件使用）
export const isDarkModeAtom = atom<boolean>((get) => {
  return get(resolvedThemeAtom) === 'dark'
})

// 从设置中获取背景模式值的atom（与后端同步）
export const appBackgroundModeAtom = atom<AppBackgroundMode>('gradient')

// Hooks
export const useThemeAtomValue = () => useAtomValue(appThemeAtom)
export const useResolvedTheme = () => useAtomValue(resolvedThemeAtom)
export const useIsDarkMode = () => useAtomValue(isDarkModeAtom)
export const useTheme = () => useAtom(appThemeAtom)
export const useBackgroundMode = () => useAtom(appBackgroundModeAtom)
export const useBackgroundModeValue = () => useAtomValue(appBackgroundModeAtom)

// 设置主题的hook（会同步到后端）
export const useSetTheme = () => {
  return async (theme: AppTheme) => {
    // 立即更新本地atom
    jotaiStore.set(appThemeAtom, theme)
    
    // 同步到后端设置
    try {
      // 强制断言为后端可接受的key，避免类型不匹配阻塞现有实现
      setGeneral('theme' as any, theme as any)
    } catch (e) {
      console.warn('Failed to save theme to backend:', e)
    }
  }
}

// 设置背景模式的hook（会同步到后端）
export const useSetBackgroundMode = () => {
  return async (mode: AppBackgroundMode) => {
    // 立即更新本地atom
    jotaiStore.set(appBackgroundModeAtom, mode)
    
    // 持久化到 UI 设置域
    try {
      setUISetting('appBackgroundMode' as any, mode as any)
    } catch (e) {
      console.warn('Failed to save background mode to backend:', e)
    }
  }
}

// 主题同步逻辑：从设置系统同步主题到主题atom
export function useThemeSync() {
  const themeFromSettings = useGeneralSettingKey('theme' as any) as 'light' | 'dark' | 'system' | undefined

  // 初始同步
  useEffect(() => {
    if (themeFromSettings) {
      jotaiStore.set(appThemeAtom, themeFromSettings)
    }
  }, [themeFromSettings])

  // 监听后端主题变化
  useEffect(() => {
    const unlisten = listenGeneral((key, value) => {
      if ((key as any) === 'theme') {
        jotaiStore.set(appThemeAtom, (value as any) || 'system')
      }
    })
    return unlisten
  }, [])
}
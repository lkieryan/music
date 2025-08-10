import { atom, useAtom, useAtomValue, useSetAtom } from 'jotai'
import { useEffect } from 'react'
import { useGeneralSettingKey, setGeneral, listenGeneral } from './general'
import { jotaiStore } from '~/lib/jotai'

export type AppTheme = 'light' | 'dark' | 'system'

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

// 检查是否为暗色模式（供渐变生成器等组件使用）
export const isDarkModeAtom = atom<boolean>((get) => {
  return get(resolvedThemeAtom) === 'dark'
})

// Hooks
export const useThemeAtomValue = () => useAtomValue(appThemeAtom)
export const useResolvedTheme = () => useAtomValue(resolvedThemeAtom)
export const useIsDarkMode = () => useAtomValue(isDarkModeAtom)
export const useTheme = () => useAtom(appThemeAtom)

// 设置主题的hook（会同步到后端）
export const useSetTheme = () => {
  return async (theme: AppTheme) => {
    // 立即更新本地atom
    jotaiStore.set(appThemeAtom, theme)
    
    // 同步到后端设置
    try {
      setGeneral('theme', theme)
    } catch (e) {
      console.warn('Failed to save theme to backend:', e)
    }
  }
}

// 主题同步逻辑：从设置系统同步主题到主题atom
export function useThemeSync() {
  const themeFromSettings = useGeneralSettingKey('theme') as 'light' | 'dark' | 'system' | undefined

  // 初始同步
  useEffect(() => {
    if (themeFromSettings) {
      jotaiStore.set(appThemeAtom, themeFromSettings)
    }
  }, [themeFromSettings])

  // 监听后端主题变化
  useEffect(() => {
    const unlisten = listenGeneral((key, value) => {
      if (key === 'theme') {
        jotaiStore.set(appThemeAtom, value || 'system')
      }
    })
    return unlisten
  }, [])
}
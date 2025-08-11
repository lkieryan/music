import { PropsWithChildren, useEffect } from 'react'
import { useAtomValue } from 'jotai'
import { resolvedThemeAtom, useThemeSync } from '~/atoms/settings/themes'

export function AppThemeProvider({ children }: PropsWithChildren) {
  // 同步设置到主题atom
  useThemeSync()
  
  const resolvedTheme = useAtomValue(resolvedThemeAtom)

  useEffect(() => {
    const root = document.documentElement
    
    // 只设置data-theme属性（你的CSS使用这个）
    root.setAttribute('data-theme', resolvedTheme)
    
    // 同时设置类名（为了兼容性）
    root.classList.remove('light', 'dark')
    root.classList.add(resolvedTheme)
  }, [resolvedTheme])

  // 监听系统主题变化
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    
    const handleChange = () => {
      // 触发 resolvedThemeAtom 重新计算
      // jotai 会自动检测到依赖变化
    }
    
    mediaQuery.addEventListener('change', handleChange)
    
    return () => {
      mediaQuery.removeEventListener('change', handleChange)
    }
  }, [])

  return <>{children}</>
}
import { useEffect, useMemo, useState } from 'react'
import { useAtom } from 'jotai'
import { activeThemeAtom, activeThemeIdAtom } from '~/providers/theme'
import { loadAllThemes, loadTheme, getThemeCss } from '~/services/theme'
import { savePreference, loadPreference } from '~/services/preferences'

export function Component() {
  const [theme, setTheme] = useAtom(activeThemeAtom)
  const [themeId, setThemeId] = useAtom(activeThemeIdAtom)
  const [all, setAll] = useState<Record<string, any>>({})
  const ids = useMemo(() => Object.keys(all || {}), [all])

  useEffect(() => {
    ;(async () => {
      try {
        const saved = await loadPreference<string>('prefs.themes.active_theme').catch(() => undefined)
        const list = await loadAllThemes().catch(() => ({}))
        setAll(list)
        if (saved && list[saved]) {
          setThemeId(saved)
          const t = await loadTheme(saved).catch(() => null)
          if (t) setTheme(t)
          const css = await getThemeCss(saved).catch(() => '')
          const el = document.querySelector('style[data-theme-css="true"]') as HTMLStyleElement | null
          if (el) el.textContent = css
        }
      } catch {}
    })()
  }, [setTheme, setThemeId])

  const onChange = async (id: string) => {
    setThemeId(id)
    await savePreference('prefs.themes.active_theme', id)
    const t = await loadTheme(id).catch(() => null)
    if (t) setTheme(t)
    const css = await getThemeCss(id).catch(() => '')
    const el = document.querySelector('style[data-theme-css="true"]') as HTMLStyleElement | null
    if (el) el.textContent = css
  }

  return (
    <div style={{ padding: 16 }}>
      <h2 style={{ fontSize: 18, fontWeight: 600, marginBottom: 12 }}>主题设置</h2>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <label htmlFor="theme-select">选择主题：</label>
        <select id="theme-select" value={themeId} onChange={(e) => onChange(e.target.value)}>
          {ids.map((id) => (
            <option key={id} value={id}>{all[id]?.meta?.name || id}</option>
          ))}
        </select>
      </div>
    </div>
  )
}

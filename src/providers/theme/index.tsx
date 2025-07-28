import { PropsWithChildren, useEffect, useRef } from 'react'
import { atom, useAtom } from 'jotai'
import type { ThemeDetails, BackgroundLayer, BackgroundConfig } from '~/types/theme'
import { loadAllThemes, loadTheme, getThemeCss } from '~/services/theme'

// Current active theme id
export const activeThemeIdAtom = atom<string>('default')

// Current theme data (loaded from backend)
export const activeThemeAtom = atom<ThemeDetails | null>(null)

function serializeLayer(layer: BackgroundLayer): string {
  const base = (s?: string) => (s && s.length ? s : undefined)
  switch (layer.type) {
    case 'linear': {
      const angle = layer.angle ?? 180
      const stops = layer.stops.map(s => `${s.color}${s.pos !== undefined ? ` ${s.pos}%` : ''}`).join(', ')
      return `linear-gradient(${angle}deg, ${stops})`
    }
    case 'radial': {
      const shape = base(layer.shape) ?? 'circle'
      const at = base(layer.at) ? ` at ${layer.at}` : ''
      const stops = layer.stops.map(s => `${s.color}${s.pos !== undefined ? ` ${s.pos}%` : ''}`).join(', ')
      return `radial-gradient(${shape}${at}, ${stops})`
    }
    case 'conic': {
      const angle = layer.angle ?? 0
      const at = base(layer.at) ? ` at ${layer.at}` : ''
      const stops = layer.stops.map(s => `${s.color}${s.pos !== undefined ? ` ${s.pos}%` : ''}`).join(', ')
      return `conic-gradient(from ${angle}deg${at}, ${stops})`
    }
    case 'image': {
      // image layer: apply position/size/repeat via shorthand if present
      const url = `url('${layer.url}')`
      // Let container CSS control repeat/size/position generally. Advanced blending could be done with pseudo-elements.
      return url
    }
  }
}

function serializeBackground(cfg?: BackgroundConfig): string | undefined {
  if (!cfg || !cfg.layers || cfg.layers.length === 0) return undefined
  const parts = cfg.layers.map(serializeLayer)
  return parts.join(', ')
}

function applyTokens(root: HTMLElement, theme: ThemeDetails) {
  const t = theme.tokens
  const set = (k: string, v?: string) => v !== undefined && root.style.setProperty(k, v)
  set('--color-bg', t.color_bg)
  set('--color-surface', t.color_surface)
  set('--color-border', t.color_border)
  set('--color-divider', t.color_divider)
  set('--text-primary', t.text_primary)
  set('--text-secondary', t.text_secondary)
  set('--text-tertiary', t.text_tertiary)
  set('--text-disabled', t.text_disabled)
  set('--accent', t.accent)
  set('--on-accent', t.on_accent)
  set('--state-hover-bg', t.state_hover_bg)
  set('--state-pressed-bg', t.state_pressed_bg)
  set('--state-selected-bg', t.state_selected_bg)
  set('--focus-ring', t.focus_ring)
  set('--shadow-1', t.shadow_1)
  if (t.colors_border) set('--colors-border', t.colors_border)
  if (t.menu_accent) set('--menu-accent', t.menu_accent)
}

function applyBackground(root: HTMLElement, theme: ThemeDetails) {
  const app = theme.background?.app
  const toolbar = theme.background?.toolbar
  const appBg = serializeBackground(app)
  const toolbarBg = serializeBackground(toolbar)
  if (appBg) root.style.setProperty('--main-browser-background', appBg)
  if (toolbarBg) root.style.setProperty('--main-browser-background-toolbar', toolbarBg)
  if (app?.opacity !== undefined) root.style.setProperty('--background-opacity', String(app.opacity))
  if (app?.grain_opacity !== undefined) root.style.setProperty('--grainy-background-opacity', String(app.grain_opacity))
  if (toolbar?.blur !== undefined) root.style.setProperty('--bg-toolbar-blur', `${toolbar.blur}px`)
}

export function ThemeProvider({ children }: PropsWithChildren) {
  const [theme, setTheme] = useAtom(activeThemeAtom)
  const [themeId, setThemeId] = useAtom(activeThemeIdAtom)
  const styleRef = useRef<HTMLStyleElement | null>(null)

  // On mount: try to load themes from backend and apply default, and wire theme-updated
  useEffect(() => {
    ;(async () => {
      try {
        const all = await loadAllThemes().catch(() => ({} as Record<string, ThemeDetails>))
        const key = themeId in all ? themeId : Object.keys(all)[0] || 'default'
        setThemeId(key)
        const t = await loadTheme(key).catch(() => null)
        if (t) setTheme(t)
        // load custom css
        const css = await getThemeCss(key).catch(() => '')
        if (!styleRef.current) {
          const el = document.createElement('style')
          el.setAttribute('data-theme-css', 'true')
          document.head.appendChild(el)
          styleRef.current = el
        }
        if (styleRef.current) styleRef.current.textContent = css
      } catch (e) {
        console.warn('Theme load failed', e)
      }
    })()
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Listen to theme-updated events to hot-reload css
  useEffect(() => {
    // @ts-ignore
    const w: any = typeof window !== 'undefined' ? window : {}
    const core = w.__TAURI__?.event
    if (!core?.listen) return
    let unlisten: any
    core.listen('theme-updated', async (e: any) => {
      const id = typeof e?.payload === 'string' ? e.payload : themeId
      if (id !== (theme?.meta?.id ?? themeId)) return
      const css = await getThemeCss(id).catch(() => '')
      if (styleRef.current) styleRef.current.textContent = css
    }).then((fn: any) => { unlisten = fn })
    return () => { if (unlisten) unlisten() }
  }, [theme, themeId])

  useEffect(() => {
    const root = document.documentElement
    root.setAttribute('data-theme', theme?.meta?.id ?? themeId)
    if (theme) {
      applyTokens(root, theme)
      applyBackground(root, theme)
    }
  }, [theme, themeId])

  return children as any
}

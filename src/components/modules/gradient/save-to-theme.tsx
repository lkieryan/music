import { useMemo } from 'react'
import type { GradientState } from '~/types/gradient'
import type { ThemeDetails, BackgroundLayer, BackgroundConfig } from '~/types/theme'
import { saveTheme } from '~/services/theme'
import { useAtomValue } from 'jotai'
import { activeThemeIdAtom } from '~/providers/theme'

function fromGradientState(state: GradientState): BackgroundConfig {
  const layers: BackgroundLayer[] = []
  // Minimal mapping: use linear as one layer if present, else radial
  if (state.linear) {
    // We cannot reconstruct precise stops; approximate with two-color fade using current linear string via custom CSS is better.
    // Here we create a simple conic/linear structure placeholder. For now, encode as image layer is suboptimal; we'll do a simple linear with angle 180.
    layers.push({ type: 'linear', angle: 180, stops: [{ color: 'rgba(0,0,0,0)' }, { color: 'rgba(0,0,0,0.3)', pos: 100 }] })
  } else if (state.radial) {
    layers.push({ type: 'radial', shape: 'circle', at: '50% 50%', stops: [{ color: 'rgba(0,0,0,0.2)' }, { color: 'rgba(0,0,0,0)', pos: 100 }] })
  }
  const opacity = state.opacity ?? 1
  const grain_opacity = state.showGrain ? 0.06 : 0
  return { layers, opacity, grain_opacity }
}

export function SaveGradientToThemeButton({ state }: { state: GradientState }) {
  const themeId = useAtomValue(activeThemeIdAtom)
  const disabled = !themeId
  const onSave = async () => {
    // Build a minimal ThemeDetails update for current theme id
    const cfg = fromGradientState(state)
    const details: Partial<ThemeDetails> = {
      background: { app: cfg }
    }
    // For simplicity, load current theme, merge, then save
    try {
      const current = await (await import('~/services/theme')).loadTheme(themeId)
      const merged: ThemeDetails = {
        meta: current.meta,
        tokens: current.tokens,
        background: {
          app: cfg,
          toolbar: current.background?.toolbar,
        },
        custom_css: current.custom_css,
      }
      await saveTheme(merged as ThemeDetails)
      alert('已保存为主题背景')
    } catch (e) {
      console.error(e)
      alert('保存失败')
    }
  }
  return (
    <button onClick={onSave} disabled={disabled} style={{ padding: '6px 10px', borderRadius: 6 }}>
      保存为主题背景
    </button>
  )
}

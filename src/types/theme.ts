// TS mirrors of Rust types (crates/types/src/themes.rs)
export type ThemeName = string

export interface ThemeMeta {
  id: string
  name: string
  author?: string
  is_dark?: boolean
  version?: string
}

export interface ThemeTokens {
  color_bg: string
  color_surface: string
  color_border: string
  color_divider: string

  text_primary: string
  text_secondary: string
  text_tertiary: string
  text_disabled: string

  accent: string
  on_accent: string

  state_hover_bg: string
  state_pressed_bg: string
  state_selected_bg: string

  focus_ring: string
  shadow_1: string

  colors_border?: string
  menu_accent?: string
}

export type LayerType = 'linear' | 'radial' | 'conic' | 'image'

export interface ColorStop { color: string; pos?: number }

export interface BackgroundLayerBase {
  opacity?: number
  blend_mode?: string
  position?: string
  size?: string
  repeat?: string
}

export interface LinearLayer extends BackgroundLayerBase { type: 'linear'; angle?: number; stops: ColorStop[] }
export interface RadialLayer extends BackgroundLayerBase { type: 'radial'; shape?: string; at?: string; stops: ColorStop[] }
export interface ConicLayer  extends BackgroundLayerBase { type: 'conic'; angle?: number; at?: string; stops: ColorStop[] }
export interface ImageLayer  extends BackgroundLayerBase { type: 'image'; url: string }

export type BackgroundLayer = LinearLayer | RadialLayer | ConicLayer | ImageLayer

export interface BackgroundConfig { layers: BackgroundLayer[]; opacity?: number; blur?: number; grain_opacity?: number }
export interface ThemeBackground { app: BackgroundConfig; toolbar?: BackgroundConfig }

export interface ThemeDetails { meta: ThemeMeta; tokens: ThemeTokens; background: ThemeBackground; custom_css?: string }

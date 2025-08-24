export interface GradientState {
  radial: string
  linear: string
  opacity: number
  texture: number
  showGrain: boolean
  // Browser parity extras
  shouldBeDark?: boolean
  toolboxTextColor?: string
}

export interface ColorHarmony {
  type: 'complementary' | 'splitComplementary' | 'analogous' | 'triadic' | 'floating'
  angles: readonly number[]
}

export interface ColorDot {
  ID: number
  c: number[] | string
  isCustom?: boolean
  algorithm?: string
  isPrimary?: boolean
  lightness?: number
  position?: { x: number; y: number }
  type?: string
}

export interface GradientInternalState {
  opacity?: number
  texture?: number
  showGrain?: boolean
  dots?: ColorDot[]
  useAlgo?: string
  currentLightness?: number
  isDarkMode?: boolean
  colorPage?: number
  customColors?: string[]
}

export interface GradientGeneratorDialogProps {
  open: boolean
  onClose: () => void
  onChange?: (data: GradientState) => void
  onInternalStateChange?: (data: GradientInternalState) => void
  initialState?: GradientInternalState
  disabled?: boolean
}

export interface PresetData {
  lightness: number
  algo: string
  numDots: number
  position: string
  style?: string
  colors?: string[]
}
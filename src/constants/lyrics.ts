export type LyricPlayerImplementationValue = "dom" | "dom-slim" | "canvas"

export const LYRIC_PLAYER_IMPLEMENTATION_VALUES: LyricPlayerImplementationValue[] = [
  "dom",
  "dom-slim",
  "canvas",
]

export function buildLyricPlayerImplementationOptions(
  t: (k: string, def?: string) => string,
) {
  return [
    {
      label: t("lyrics.appearance.player_implementation.dom", "DOM"),
      value: "dom" as LyricPlayerImplementationValue,
    },
    {
      label: t("lyrics.appearance.player_implementation.dom-slim", "DOM (Stripped)"),
      value: "dom-slim" as LyricPlayerImplementationValue,
    },
    {
      label: t("lyrics.appearance.player_implementation.canvas", "Canvas"),
      value: "canvas" as LyricPlayerImplementationValue,
    },
  ]
}

export type LyricSizePresetValue =
  | "tiny"
  | "extra_small"
  | "small"
  | "medium"
  | "large"
  | "extra_large"
  | "huge"

export const LYRIC_SIZE_PRESET_VALUES: LyricSizePresetValue[] = [
  "tiny",
  "extra_small",
  "small",
  "medium",
  "large",
  "extra_large",
  "huge",
]

export function buildLyricSizePresetOptions(t: (k: string, def?: string) => string) {
  return [
    { label: t("lyrics.appearance.size_preset.tiny", "Tiny"), value: "tiny" as LyricSizePresetValue },
    { label: t("lyrics.appearance.size_preset.extra_small", "Extra Small"), value: "extra_small" as LyricSizePresetValue },
    { label: t("lyrics.appearance.size_preset.small", "Small"), value: "small" as LyricSizePresetValue },
    { label: t("lyrics.appearance.size_preset.medium", "Medium"), value: "medium" as LyricSizePresetValue },
    { label: t("lyrics.appearance.size_preset.large", "Large"), value: "large" as LyricSizePresetValue },
    { label: t("lyrics.appearance.size_preset.extra_large", "Extra Large"), value: "extra_large" as LyricSizePresetValue },
    { label: t("lyrics.appearance.size_preset.huge", "Huge"), value: "huge" as LyricSizePresetValue },
  ]
}

export type LyricBackgroundRendererValue = "mesh" | "pixi" | "css-bg"

export const LYRIC_BACKGROUND_RENDERER_VALUES: LyricBackgroundRendererValue[] = [
  "mesh",
  "pixi",
  "css-bg",
]

export function buildLyricBackgroundRendererOptions(
  t: (k: string, def?: string) => string,
) {
  return [
    {
      label: t(
        "lyrics.background.renderer.mesh_gradient",
        "Mesh Gradient Renderer",
      ),
      value: "mesh" as LyricBackgroundRendererValue,
    },
    {
      label: t("lyrics.background.renderer.pixi", "PixiJS Renderer"),
      value: "pixi" as LyricBackgroundRendererValue,
    },
    {
      label: t("lyrics.background.renderer.css_bg", "CSS Background"),
      value: "css-bg" as LyricBackgroundRendererValue,
    },
  ]
}



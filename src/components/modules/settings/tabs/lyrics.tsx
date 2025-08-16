import { useTranslation } from "react-i18next"
import { useState } from "react"
import { createSetting } from "../helper/builder"
import {
  setLyricsSetting,
  setLyrics,
  useLyricsSettingValue,
} from "~/atoms/settings/lyrics"
import { SettingItemGroup } from "../section"
import { SettingDescription, SettingInput, SettingSwitch } from "../control"
import { ResponsiveSelect } from "~/components/ui/select/responsive"
import { 
  buildLyricPlayerImplementationOptions,
  buildLyricSizePresetOptions,
  buildLyricBackgroundRendererOptions,
} from "~/constants/lyrics"
import { Input } from "~/components/ui/input"

const { SettingBuilder, defineSettingItem } = createSetting(
  useLyricsSettingValue,
  setLyricsSetting,
  (k, v) => setLyrics(k as any, v as any),
)

export const SettingLyrics = () => {
  const { t } = useTranslation("settings")

  return (
    <div className="mt-4">
      <SettingBuilder
        key="lyrics-settings"
        settings={[
          { type: "title", value: t("lyrics.content") },
          defineSettingItem("translationLine", { 
            label: t("lyrics.content.translation_line"),
            persist: true,
          }),
          defineSettingItem("romanLine", { 
            label: t("lyrics.content.roman_line"),
            persist: true,
          }),
          defineSettingItem("swapTransRomanLine", { 
            label: t("lyrics.content.swap_trans_roman_line"),
            description: t("lyrics.content.swap_trans_roman_line.description"),
            persist: true,
          }),

          { type: "title", value: t("lyrics.appearance") },
          PlayerImplementationItem,
          defineSettingItem("fontFamily", { 
            label: t("lyrics.appearance.font_family"),
            description: t("lyrics.appearance.font_family.description"),
            persist: true,
            componentProps: { inputClassName: "w-48" },
          }),
          FontWeightItem,
          defineSettingItem("letterSpacing", { 
            label: t("lyrics.appearance.letter_spacing"),
            description: t("lyrics.appearance.letter_spacing.description"),
            persist: true,
            componentProps: { inputClassName: "w-48" },
          }),
          FontPreviewItem,
          
          SizePresetItem,
          defineSettingItem("lineBlurEffect", { 
            label: t("lyrics.appearance.line_blur_effect"),
            description: t("lyrics.appearance.line_blur_effect.description"),
            persist: true,
          }),
          defineSettingItem("lineScaleEffect", 
            { label: t("lyrics.appearance.line_scale_effect"),
            description: t("lyrics.appearance.line_scale_effect.description"),
            persist: true,
          }),
          defineSettingItem("lineSpringAnimation", { 
            label: t("lyrics.appearance.line_spring_animation"),
            description: t("lyrics.appearance.line_spring_animation.description"),
            persist: true,
          }),
          defineSettingItem("advanceLineTiming", { 
            label: t("lyrics.appearance.advance_line_timing"),
            description: t("lyrics.appearance.advance_line_timing.description"),
            persist: true,
          }),
          WordFadeWidthItem,

          { type: "title", value: t("lyrics.background") },
          BackgroundRendererItem,
          BackgroundConditionalItem,
        ]}
      />
    </div>
  )
}

const FontWeightItem = () => {
  const { t } = useTranslation("settings")
  const settings = useLyricsSettingValue()
  return (
    <SettingItemGroup>
      <SettingInput
        type="number"
        label={t("lyrics.appearance.font_weight")}
        value={String(Number(settings.fontWeight || 0))}
        onChange={(e) => {
          const n = Math.max(0, Math.min(1000, Number(e.target.value) || 0))
          setLyrics("fontWeight", String(n))
        }}
        inputClassName="w-48"
      />
      <SettingDescription>
        {t("lyrics.appearance.font_weight.description")}
      </SettingDescription>
    </SettingItemGroup>
  )
}



const PlayerImplementationItem = () => {
  const { t } = useTranslation("settings")
  const settings = useLyricsSettingValue()
  const items = buildLyricPlayerImplementationOptions(t)
  const value = (settings.playerImplementation as string) || "dom"
  return (
    <SettingItemGroup>
      <div className="mb-3 flex items-center justify-between gap-4">
        <label className="text-sm font-medium leading-none">
          {t("lyrics.appearance.player_implementation")}
        </label>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={value}
          onValueChange={(v) => setLyrics("playerImplementation", v as any)}
          items={items}
        />
      </div>
      <SettingDescription>
        {t("lyrics.appearance.player_implementation.description")}
      </SettingDescription>
    </SettingItemGroup>
  )
}

const SizePresetItem = () => {
  const { t } = useTranslation("settings")
  const settings = useLyricsSettingValue()
  const items = buildLyricSizePresetOptions(t)
  const value = (settings.sizePreset as string) || "medium"
  return (
    <SettingItemGroup>
      <div className="mb-3 flex items-center justify-between gap-4">
        <label className="text-sm font-medium leading-none">
          {t("lyrics.appearance.size_preset")}
        </label>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={value}
          onValueChange={(v) => setLyrics("sizePreset", v as any)}
          items={items}
        />
      </div>
      <SettingDescription>
        {t("lyrics.appearance.size_preset.description")}
      </SettingDescription>
    </SettingItemGroup>
  )
}


const FontPreviewItem = () => {
  const { t } = useTranslation("settings")
  const settings = useLyricsSettingValue()
  const [preview, setPreview] = useState("Hello, 世界 – かなカナ")

  const fontFamily = settings.fontFamily || undefined
  const fontWeight = settings.fontWeight || undefined
  const letterSpacing = (settings.letterSpacing as any) || undefined

  return (
    <SettingItemGroup>
      <div className="flex items-center gap-4 my-2 flex-wrap">
        <div className="flex-1 min-w-40 text-sm font-medium">
          {t("lyrics.appearance.font_preview")}
        </div>
        <div className="w-48">
          <Input
            value={preview}
            onChange={(e) => setPreview(e.currentTarget.value)}
            className="h-8 px-2 text-xs border border-border rounded-md bg-background text-text"
          />
        </div>
      </div>
      <div
        className="text-center"
        style={{
          fontFamily,
          fontWeight: fontWeight as any,  
          letterSpacing: letterSpacing as any,
          fontSize: "max(max(4.7vh, 3.2vw), 12px)",
        }}
      >
        {preview}
        <div style={{ fontSize: "max(0.5em, 10px)", opacity: 0.3 }}>{preview}</div>
      </div>
    </SettingItemGroup>
  )
}

const WordFadeWidthItem = () => {
  const { t } = useTranslation("settings")
  const settings = useLyricsSettingValue()
  return (
    <SettingItemGroup>
      <SettingInput
        type="text"
        label={t("lyrics.appearance.word_fade_width")}
        value={String(Number(settings.wordFadeWidth || 0))}
        onChange={(e) => {
        const n = Math.max(0, Math.min(1000, Number(e.target.value) || 0))
        setLyrics("wordFadeWidth", n)
        }}
        inputClassName="w-48"
      />
      <SettingDescription>
        {t("lyrics.appearance.word_fade_width.description")}
      </SettingDescription>
    </SettingItemGroup>
  )
}


const BackgroundConditionalItem = () => {
  const { t } = useTranslation("settings")
  const settings = useLyricsSettingValue()
  const isCss = settings.backgroundRenderer === "css-bg"

  if (isCss) {
    return (
      <SettingItemGroup>
        <SettingInput
          type="text"
          label={t("lyrics.background.css_background_property")}
          value={settings.cssBackgroundProperty || ""}
          onChange={(e) => setLyricsSetting("cssBackgroundProperty", e.target.value)}
          inputClassName="w-48"
        />
        <SettingDescription>
          {t("lyrics.background.css_background_property.description")}
        </SettingDescription>
      </SettingItemGroup>
    )
  }

  return (
    <>
      <SettingItemGroup>
        <SettingInput
          type="number"
          label={t("lyrics.background.fps")}
          value={String(settings.backgroundFps ?? 60)}
          onChange={(e) => setLyricsSetting("backgroundFps", Number(e.target.value))}
          inputClassName="w-48"
        />
        <SettingDescription>
          {t("lyrics.background.fps.description")}
        </SettingDescription>
      </SettingItemGroup>

      <SettingItemGroup>
        <SettingInput
          type="number"
          label={t("lyrics.background.render_scale")}
          value={String(settings.backgroundRenderScale ?? 1)}
          onChange={(e) => setLyricsSetting("backgroundRenderScale", Number(e.target.value))}
          inputClassName="w-48"
        />
        <SettingDescription>
          {t("lyrics.background.render_scale.description")}
        </SettingDescription>
      </SettingItemGroup>

      <SettingItemGroup>
        <SettingSwitch
          label={t("lyrics.background.static_mode")}
          checked={Boolean(settings.backgroundStaticMode)}
          onCheckedChange={(v) => setLyricsSetting("backgroundStaticMode", Boolean(v))}
        />
        <SettingDescription>
          {t("lyrics.background.static_mode.description")}
        </SettingDescription>
      </SettingItemGroup>
    </>
  )
}

const BackgroundRendererItem = () => {
  const { t } = useTranslation("settings")
  const settings = useLyricsSettingValue()
  const items = buildLyricBackgroundRendererOptions(t)
  const value = (settings.backgroundRenderer as string) || "mesh"
  return (
    <SettingItemGroup>
      <div className="mb-3 flex items-center justify-between gap-4">
        <label className="text-sm font-medium leading-none">
          {t("lyrics.background.renderer")}
        </label>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={value}
          onValueChange={(v) => setLyrics("backgroundRenderer", v as any)}
          items={items}
        />
      </div>
      <SettingDescription>
        {t("lyrics.background.renderer.description")}
      </SettingDescription>
    </SettingItemGroup>
  )
}



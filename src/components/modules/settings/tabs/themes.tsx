
import {
  setGeneralSetting,
  useGeneralSettingValue,
} from "~/atoms/settings/general"
import { useThemeAtomValue, useSetTheme } from "~/atoms/settings/themes"
import { useAtom } from "jotai"
import {
  toolbarModeAtom,
  sidebarPositionAtom,
  playerPlacementAtom,
  playerVisibleAtom,
  playerHeightAtom,
  type ToolbarMode,
  type SidebarPosition,
  type PlayerPlacement
} from "~/atoms/layout"
import { createSetting } from "../helper/builder"
import { useTranslation } from "react-i18next"
import {
  SettingTabbedSegment,
  SettingSwitch,
  SettingActionItem,
  SettingInput,
} from "../control"
import { SettingItemGroup } from "../section"
import { ResponsiveSelect } from "~/components/ui/select/responsive"
import { useState, useCallback } from "react"
import GradientGeneratorDialog from "~/components/gradient/index"
import { useBackgroundModeValue, useSetBackgroundMode, useBackgroundSettingsValue, setBackgroundSetting } from "~/atoms/settings/themes"
import { SettingDescription } from "../control"
import { buildLyricBackgroundRendererOptions } from "~/constants/lyrics"



const { SettingBuilder } = createSetting(
  useGeneralSettingValue,
  setGeneralSetting,
)

export const SettingThemes = () => {
  const { t } = useTranslation("settings")
  const bgMode = useBackgroundModeValue()
  return (
    <div className="mt-4">
      <SettingBuilder
        settings={[
          {
            type: "title",
            value: t("themes.label"),
          },
          AppThemeSegment,
          BackgroundModeSegment,
          ...(bgMode === 'gradient' ? [BackgroundSetting] as const : []),
          ...(bgMode === 'dynamic_cover'
            ? ([
                BackgroundRendererItem,
                BackgroundConditionalItem,
              ] as const)
            : []),
          {
            type: "title",
            value: t("themes.layout"),
          },
          SidebarPositionSegment,
          ToolbarModeSegment,
          PlayerVisibleSetting,
          PlayerPlacementSegment,
          PlayerHeightSetting,
        ]}
      />
    </div>
  )
}

// Map UI display value to internal theme and apply via useSetTheme
const useApplyThemeDisplayValue = () => {
  const setTheme = useSetTheme()
  return async (value: string) => {
    const themeMap = {
      'auto': 'system',
      'light': 'light',
      'dark': 'dark',
    } as const
    await setTheme(themeMap[value as keyof typeof themeMap] as any)
  }
}

// ================= Lyrics Background Settings (moved from lyrics tab) =================
const BackgroundConditionalItem = () => {
  const { t } = useTranslation("settings")
  const settings = useBackgroundSettingsValue()
  const isCss = settings.backgroundRenderer === "css-bg"

  if (isCss) {
    return (
      <SettingItemGroup>
        <SettingInput
          type="text"
          label={t("lyrics.background.css_background_property")}
          value={settings.cssBackgroundProperty || ""}
          onChange={(e) => setBackgroundSetting("cssBackgroundProperty", e.target.value)}
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
          onChange={(e) => setBackgroundSetting("backgroundFps", Number(e.target.value))}
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
          onChange={(e) => setBackgroundSetting("backgroundRenderScale", Number(e.target.value))}
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
          onCheckedChange={(v) => setBackgroundSetting("backgroundStaticMode", Boolean(v))}
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
  const settings = useBackgroundSettingsValue()
  const items = buildLyricBackgroundRendererOptions(t as any)
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
          onValueChange={(v) => setBackgroundSetting("backgroundRenderer", v as any)}
          items={items}
        />
      </div>
      <SettingDescription>
        {t("lyrics.background.renderer.description")}
      </SettingDescription>
    </SettingItemGroup>
  )
}

export const AppThemeSegment = () => {
  const { t } = useTranslation("settings")
  const theme = useThemeAtomValue()
  const applyThemeDisplay = useApplyThemeDisplayValue()

  // 使用与 SchemeButtons 相同的映射逻辑
  const currentDisplayValue = theme === 'system' ? 'auto' : theme

  const handleThemeChange = async (value: string) => {
    await applyThemeDisplay(value)
  }

  return (
    <SettingTabbedSegment
      key="theme"
      label={t("themes.mode")}
      value={currentDisplayValue}
      values={[
        {
          value: "auto",
          label: t("themes.mode.system"),
          icon: <i className="i-mingcute-monitor-line" />,
        },
        {
          value: "light",
          label: t("themes.mode.light"),
          icon: <i className="i-mingcute-sun-line" />,
        },
        {
          value: "dark",
          label: t("themes.mode.dark"),
          icon: <i className="i-mingcute-moon-line" />,
        },
      ]}
      onValueChanged={handleThemeChange}
    />
  )
}

export const ToolbarModeSegment = () => {
  const { t } = useTranslation("settings")
  const [toolbarMode, setToolbarMode] = useAtom(toolbarModeAtom)

  return (
    <SettingItemGroup>
      <div className="mb-3 mt-4 flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t("themes.toolbar.label")}</span>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={toolbarMode}
          onValueChange={(value) => {
            setToolbarMode(value as ToolbarMode)
          }}
          items={[
            {
              label: t("themes.toolbar.single"),
              value: "single",
            },
            {
              label: t("themes.toolbar.multi"),
              value: "multi",
            },
            {
              label: t("themes.toolbar.compact"),
              value: "compact",
            },
          ]}
        />
      </div>
    </SettingItemGroup>
  )
}

export const SidebarPositionSegment = () => {
  const { t } = useTranslation("settings")
  const [sidebarPosition, setSidebarPosition] = useAtom(sidebarPositionAtom)

  return (
    <SettingTabbedSegment
      key="sidebarPosition"
      label={t("themes.sidebar.label")}
      value={sidebarPosition}
      values={[
        {
          value: "left",
          label: t("themes.sidebar.left"),
          icon: <i className="i-mingcute-align-left-line" />,
        },
        {
          value: "right",
          label: t("themes.sidebar.right"),
          icon: <i className="i-mingcute-align-right-line" />,
        },
      ]}
      onValueChanged={(value) => {
        setSidebarPosition(value as SidebarPosition)
      }}
    />
  )
}

export const PlayerPlacementSegment = () => {
  const { t } = useTranslation("settings")
  const [playerPlacement, setPlayerPlacement] = useAtom(playerPlacementAtom)

  return (
    <SettingItemGroup>
      <div className="mb-3 mt-4 flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t("themes.player.label")}</span>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={playerPlacement}
          onValueChange={(value) => {
            setPlayerPlacement(value as PlayerPlacement)
          }}
          items={[
            {
              label: t("themes.player.none"),
              value: "none",
            },
            {
              label: t("themes.player.global-bottom"),
              value: "global-bottom",
            },
            {
              label: t("themes.player.content-bottom"),
              value: "content-bottom",
            },
            {
              label: t("themes.player.sidebar-bottom"),
              value: "sidebar-bottom",
            },
            {
              label: t("themes.player.sidebar-middle"),
              value: "sidebar-middle",
            },
          ]}
        />
      </div>
    </SettingItemGroup>
  )
}

export const PlayerVisibleSetting = () => {
  const { t } = useTranslation("settings")
  const [playerVisible, setPlayerVisible] = useAtom(playerVisibleAtom)

  return (
    <SettingItemGroup>
      <SettingSwitch
        checked={playerVisible}
        onCheckedChange={setPlayerVisible}
        label={t("themes.player.visible")}
      />
    </SettingItemGroup>
  )
}

export const PlayerHeightSetting = () => {
  const { t } = useTranslation("settings")
  const [playerHeight, setPlayerHeight] = useAtom(playerHeightAtom)

  return (
    <SettingItemGroup>
      <SettingInput
        type="number"
        label={t("themes.player.height")}
        value={String(playerHeight)}
        onChange={(e) => {
          const value = parseInt(e.target.value) || 64
          if (value >= 40 && value <= 160) {
            setPlayerHeight(value)
          }
        }}
        inputClassName="w-48"
      />
    </SettingItemGroup>
  )
}


// Background Mode Selection (gradient | dynamic_cover) with Select
export const BackgroundModeSegment = () => {
  const { t } = useTranslation("settings")
  const value = useBackgroundModeValue()
  const setMode = useSetBackgroundMode()
  const applyThemeDisplay = useApplyThemeDisplayValue()

  return (
    <SettingItemGroup>
      <div className="mb-3 mt-4 flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t("themes.background.mode")}</span>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={value ?? 'gradient'}
          onValueChange={async (v) => {
            await setMode(v as any)
            if (v === 'dynamic_cover') {
              await applyThemeDisplay('dark')
            }
          }}
          items={[
            {
              label: t("themes.background.mode.gradient"),
              value: "gradient",
            },
            {
              label: t("themes.background.mode.dynamic_cover"),
              value: "dynamic_cover",
            },
          ]}
        />
      </div>
    </SettingItemGroup>
  )
}

export const BackgroundSetting = () => {
  const { t } = useTranslation("settings")
  const [isGradientOpen, setIsGradientOpen] = useState(false)

  // 这些回调现在是可选的，因为BackgroundProvider会处理状态
  const handleGradientChange = useCallback(() => {
    // BackgroundProvider会自动处理状态更新和DOM应用
  }, [])

  const handleInternalStateChange = useCallback(() => {
    // BackgroundProvider会自动处理内部状态更新
  }, [])

  return (
    <>
      <SettingItemGroup>
        <SettingActionItem
          label={t("themes.background.label")}
          action={() => setIsGradientOpen(true)}
          buttonText={t("themes.background.open")}
        />
      </SettingItemGroup>

      {isGradientOpen && (
        <GradientGeneratorDialog
          open={isGradientOpen}
          onClose={() => setIsGradientOpen(false)}
          onChange={handleGradientChange}
          onInternalStateChange={handleInternalStateChange}
        />
      )}
    </>
  )
}

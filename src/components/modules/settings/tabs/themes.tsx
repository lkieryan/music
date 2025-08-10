
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
} from "../control"
import { SettingItemGroup } from "../section"
import { ResponsiveSelect } from "~/components/ui/select/responsive"
import { useState, useCallback } from "react"
import GradientGeneratorDialog from "~/components/gradient/index"



const { SettingBuilder } = createSetting(
  useGeneralSettingValue,
  setGeneralSetting,
)

export const SettingThemes = () => {
  const { t } = useTranslation("settings")
  return (
    <div className="mt-4">
      <SettingBuilder
        settings={[
          {
            type: "title",
            value: t("themes.label"),
          },
          AppThemeSegment,
          BackgroundSetting,
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

export const AppThemeSegment = () => {
  const { t } = useTranslation("settings")
  const theme = useThemeAtomValue()
  const setTheme = useSetTheme()
  
  // 使用与 SchemeButtons 相同的映射逻辑
  const currentDisplayValue = theme === 'system' ? 'auto' : theme
  
  const handleThemeChange = async (value: string) => {
    const themeMap = {
      'auto': 'system',
      'light': 'light', 
      'dark': 'dark'
    } as const
    
    await setTheme(themeMap[value as keyof typeof themeMap])
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
              label: t("themes.player.global-bottom"),
              value: "global-bottom",
            },
            {
              label: t("themes.player.content-bottom"),
              value: "content-bottom",
            },
            {
              label: t("themes.player.sidebar"),
              value: "sidebar",
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
      <div className="mb-3 mt-4 flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t("themes.player.height")}</span>
        <input
          type="number"
          min={40}
          max={160}
          value={playerHeight}
          onChange={(e) => {
            const value = parseInt(e.target.value) || 64
            if (value >= 40 && value <= 160) {
              setPlayerHeight(value)
            }
          }}
          className="w-48 h-8 px-2 text-xs border border-border rounded-md bg-background text-text"
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

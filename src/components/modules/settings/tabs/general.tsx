import { cn } from "~/lib/helper"
import { open } from "@tauri-apps/plugin-dialog"
import TrashIcon from "~/assets/icons/trash.svg?react"
import FolderIcon from "~/assets/icons/folder.svg?react"
import PlusIcon from "~/assets/icons/plus.svg?react"
import {
  setGeneralSetting,
  useGeneralSettingKey,
  useGeneralSettingValue,
  setGeneral,
} from "~/atoms/settings/general"
import { createSetting } from "../helper/builder"
import { useMobile } from "~/hooks/common/use-mobile"
import { useTranslation } from "react-i18next"
import { SettingItemGroup } from "../section"
import { SettingDescription, SettingInput, SettingSwitch } from "../control"
import { ResponsiveSelect } from "~/components/ui/select/responsive"
import { currentSupportedLanguages } from "~/i18n"

const { defineSettingItem: _defineSettingItem, SettingBuilder } = createSetting(
  useGeneralSettingValue,
  setGeneralSetting,
  (k, v) => setGeneral(k as any, v as any),
)

export const SettingGeneral = () => {
  const { t } = useTranslation("settings")
  const isMobile = useMobile()

  return (
    <div className="mt-4">
      <SettingBuilder
        key="general-settings"
        settings={[
          {
            type: "title",
            value: t("general.app"),
          },
          LaunchAtLoginSetting,
          MinimizeToTraySetting,
          LanguageSelector,

          {
            type: "title",
            value: t("general.scan"),
          },
          AutoScanEnabledSetting,
          ScanFoldersSetting,
          ScanRulesSetting,
        ]}
      />
    </div>
  )
}

// Scan rules: minimal UI matching backend enums (scanMinDuration, scanFormats)
const ScanRulesSetting = () => {
  const { t } = useTranslation('settings')
  const scanMinDuration = (useGeneralSettingKey('scanMinDuration') as string) || 'sec30'
  const scanFormats = (useGeneralSettingKey('scanFormats') as string) || 'common'

  return (
    <SettingItemGroup>
      <div className="mb-3 mt-2 flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t('general.scan_min_duration.label')}</span>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={scanMinDuration}
          onValueChange={(value) => setGeneral('scanMinDuration', value as any)}
          items={[
            { label: t('general.scan_min_duration.sec30'), value: 'sec30' },
            { label: t('general.scan_min_duration.min2'), value: 'min2' },
            { label: t('general.scan_min_duration.all'), value: 'all' },
          ]}
        />
      </div>
      <SettingDescription>{t('general.scan_min_duration.description')}</SettingDescription>

      <div className="mb-1 mt-4 flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t('general.scan_formats.label')}</span>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          value={scanFormats}
          onValueChange={(value) => setGeneral('scanFormats', value as any)}
          items={[
            { label: t('general.scan_formats.common'), value: 'common' },
            { label: t('general.scan_formats.all'), value: 'all' },
          ]}
        />
      </div>
      <SettingDescription>{t('general.scan_formats.description')}</SettingDescription>
    </SettingItemGroup>
  )
}

const AutoScanEnabledSetting = () => {
  const { t } = useTranslation('settings')
  const autoScanEnabled = useGeneralSettingKey('autoScanEnabled') as boolean | undefined
  const onChange = async (checked: boolean) => {
    setGeneral('autoScanEnabled', checked)
  }
  return (
    <SettingItemGroup>
      <SettingSwitch
        checked={!!autoScanEnabled}
        className="mt-4"
        onCheckedChange={onChange}
        label={t('general.auto_scan_enabled.label')}
      />
      <SettingDescription>{t('general.auto_scan_enabled.description')}</SettingDescription>
    </SettingItemGroup>
  )
}

const ScanFoldersSetting = () => {
  const { t } = useTranslation('settings')
  const scanFolders = (useGeneralSettingKey('scanFolders') as string[] | undefined) || []

  const addFolders = async () => {
    const picked = await open({ directory: true, multiple: true })
    if (!picked) return
    const arr = Array.isArray(picked) ? picked : [picked]
    const set = new Set<string>(scanFolders)
    for (const p of arr) {
      if (typeof p === 'string' && p.trim()) set.add(p)
    }
    setGeneral('scanFolders', Array.from(set))
  }

  const removeFolder = (path: string) => {
    setGeneral('scanFolders', scanFolders.filter((p) => p !== path))
  }

  return (
    <SettingItemGroup>
      <div className="mb-2 flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t('general.scan_folders.label')}</span>
        <button
          type="button"
          onClick={addFolders}
          className="inline-flex items-center rounded-md border border-border px-2 py-1 text-xs hover:bg-accent/5"
        >
          <PlusIcon className="mr-1.5 h-3.5 w-3.5" />
          <span>{t('general.scan_folders.add')}</span>
        </button>
      </div>
      <SettingDescription>{t('general.scan_folders.description')}</SettingDescription>
      <div className="h-32 overflow-y-auto rounded-lg border border-border">
        {scanFolders.length === 0 ? (
          <div className="p-3 text-xs opacity-70">{t('general.scan_folders.empty')}</div>
        ) : (
          <ul>
            {scanFolders.map((p) => (
              <li key={p} className="group flex items-center justify-between px-3 py-1 rounded hover:bg-theme-item-hover transition-colors">
                <div className="flex min-w-0 items-center gap-2">
                  <FolderIcon className="h-4 w-4 opacity-70" />
                  <span className="truncate text-sm" title={p}>{p}</span>
                </div>
                <button
                  type="button"
                  onClick={() => removeFolder(p)}
                  aria-label="Remove folder"
                  className="ml-2 rounded p-1 text-text/70 opacity-0 transition-opacity group-hover:opacity-100 hover:text-accent"
                >
                  <TrashIcon className="h-4 w-4" />
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </SettingItemGroup>
  )
}

const MinimizeToTraySetting = () => {
  const { t } = useTranslation('settings')
  const minimizeToTray = useGeneralSettingKey('minimizeToTray') as boolean | undefined
  const onChange = async (checked: boolean) => {
    setGeneral('minimizeToTray', checked)
  }
  return (
    <SettingItemGroup>
      <SettingSwitch
        checked={!!minimizeToTray}
        className="mt-4"
        onCheckedChange={onChange}
        label={t('general.minimize_to_tray.label')}
      />
      <SettingDescription>{t('general.minimize_to_tray.description')}</SettingDescription>
    </SettingItemGroup>
  )
}

const LaunchAtLoginSetting = () => {
  const { t } = useTranslation('settings')
  const launch = useGeneralSettingKey('launchAtLogin') as boolean | undefined
  const onChange = async (checked: boolean) => {
    setGeneral('launchAtLogin', checked)
  }
  return (
    <SettingItemGroup>
      <SettingSwitch
        checked={!!launch}
        className="mt-4"
        onCheckedChange={onChange}
        label={t('general.launch_at_login')}
      />
    </SettingItemGroup>
  )
}

export const LanguageSelector = ({
  containerClassName,
  contentClassName,
}: {
  containerClassName?: string
  contentClassName?: string
}) => {
  const { t } = useTranslation('settings')
  const language = useGeneralSettingKey('language') as string

  const labelOf = (lang: string) => {
    switch (lang) {
      case 'en':
        return t('words.english', { ns: 'common' }) || 'English'
      case 'zh-CN':
        return t('words.chinese', { ns: 'common' }) || '简体中文'
      default:
        return lang
    }
  }

  return (
    <div className={cn('mb-3 mt-4 flex items-center justify-between', containerClassName)}>
      <span className="shrink-0 text-sm font-medium">{t('general.language')}</span>

      <ResponsiveSelect
        size="sm"
        triggerClassName="w-48"
        contentClassName={contentClassName}
        value={language}
        onValueChange={async (value) => {
          setGeneral('language', value)
        }}
        items={currentSupportedLanguages.map((lang) => ({
          label: labelOf(lang),
          value: lang,
        }))}
      />
    </div>
  )
}
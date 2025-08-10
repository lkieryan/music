
import { cn } from "~/lib/helper"
import {
  setGeneralSetting,
  useGeneralSettingKey,
  useGeneralSettingValue,
  useGeneralSettingSelector,
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
        ]}
      />
    </div>
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

const TranslationModeSelector = () => {
  const { t } = useTranslation("settings")
  const translationMode = useGeneralSettingKey("translationMode")

  return (
    <SettingItemGroup>
      <div className="flex items-center justify-between">
        <span className="shrink-0 text-sm font-medium">{t("general.translation_mode.label")}</span>
        <ResponsiveSelect
          size="sm"
          triggerClassName="w-48"
          defaultValue={translationMode}
          value={translationMode}
          onValueChange={(value) => {
            setGeneralSetting("translationMode", value as "bilingual" | "translation-only")
          }}
          items={[
            { label: t("general.translation_mode.bilingual"), value: "bilingual" },
            { label: t("general.translation_mode.translation-only"), value: "translation-only" },
          ]}
        />
      </div>
      <SettingDescription>{t("general.translation_mode.description")}</SettingDescription>
    </SettingItemGroup>
  )
}

const ActionLanguageSelector = () => {
  const { t } = useTranslation("settings")
  const actionLanguage = useGeneralSettingKey("actionLanguage")

  return (
    <div className="mb-3 mt-4 flex items-center justify-between">
      <span className="shrink-0 text-sm font-medium">{t("general.action_language.label")}</span>
      <ResponsiveSelect
        size="sm"
        triggerClassName="w-48"
        defaultValue={actionLanguage}
        value={actionLanguage}
        onValueChange={() => {}}
          items={[
          
        ]}
      />
    </div>
  )
}
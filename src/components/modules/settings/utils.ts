

export enum DisableWhy {
  Noop = "noop",
  NotActivation = "not_activation",
}

export interface SettingPageConfig {
  icon: string | React.ReactNode
  name: I18nKeysForSettings
  priority: number
  headerIcon?: string | React.ReactNode
  viewportClassName?: string
}
export const defineSettingPageData = (config: SettingPageConfig) => ({
  ...config,
})

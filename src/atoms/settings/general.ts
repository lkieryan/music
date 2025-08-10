import { createSettingAtom } from "~/atoms/helper/setting"
import { useCallback, useMemo } from "react"
// Strongly type backend mapping with generated bindings
import type { GeneralSettings } from "~/types/bindings"

import { jotaiStore } from "~/lib/jotai"
// Generic, domain-based binding to avoid per-key wiring
import type { DomainBinding } from '~/services/settings'
import { createDomainBinding } from '~/services/settings'

type GeneralDefaults = Strictify<GeneralSettings>

export const createDefaultGeneralSettings = (): GeneralDefaults => ({
  // Store as string locally for simple input binding; backend uses number
  gaplessSkip: "0",
  // Active UI language in renderer (BCP-47)
  language: "zh-CN",
  // App theme mode (light/dark/system)
  theme: "system" as "light" | "dark" | "system",
  // minimize to tray behavior on window close (desktop only)
  minimizeToTray: false,
  // launch app at OS login (desktop only)
  launchAtLogin: false,
})

const {
  useSettingKey: useGeneralSettingKeyInternal,
  useSettingSelector: useGeneralSettingSelectorInternal,
  useSettingKeys: useGeneralSettingKeysInternal,
  setSetting: setGeneralSetting,
  clearSettings: clearGeneralSettings,
  initializeDefaultSettings: initializeDefaultGeneralSettings,
  getSettings: getGeneralSettingsInternal,
  useSettingValue: useGeneralSettingValueInternal,
  settingAtom: __generalSettingAtom,
} = createSettingAtom("general", createDefaultGeneralSettings)

const [
  useGeneralSettingKey,
  useGeneralSettingSelector,
  useGeneralSettingKeys,
  getGeneralSettings,
  useGeneralSettingValue,
] = hookEnhancedSettings(
  useGeneralSettingKeyInternal,
  useGeneralSettingSelectorInternal,
  useGeneralSettingKeysInternal,
  getGeneralSettingsInternal,
  useGeneralSettingValueInternal,
  new Set(), // enhancedGeneralSettingKeys (not used for now)
  {}, // defaultGeneralSettings (not used for now)
)
export {
  __generalSettingAtom,
  clearGeneralSettings,
  getGeneralSettings,
  initializeDefaultGeneralSettings,
  setGeneralSetting,
  useGeneralSettingKey,
  useGeneralSettingKeys,
  useGeneralSettingSelector,
  useGeneralSettingValue,
}

let __generalBinding: DomainBinding<GeneralDefaults> | null = null

export function getGeneralBinding() {
  if (!__generalBinding) {
    __generalBinding = createDomainBinding<GeneralDefaults, GeneralSettings>({
      domain: 'general',
      defaultFactory: createDefaultGeneralSettings,
      setLocal: (k, v) => setGeneralSetting(k as any, v as any),
      keyMap: {} as any,
    })
  }
  return __generalBinding
}

export async function hydrateGeneral() {
  await getGeneralBinding().hydrate()
}

export function listenGeneral(onAfterChange?: (k: keyof GeneralDefaults, v: any) => void) {
  return getGeneralBinding().listen(onAfterChange as any)
}

export function setGeneral<K extends keyof GeneralDefaults>(key: K, value: GeneralDefaults[K]) {
  return getGeneralBinding().set(key, value)
}

export function hookEnhancedSettings<
  T1 extends (key: any) => any,
  T2 extends (selector: (s: any) => any) => any,
  T3 extends (keys: any) => any,
  T4 extends () => any,
  T5 extends () => any,
>(
  useSettingKey: T1,
  useSettingSelector: T2,
  useSettingKeys: T3,
  getSettings: T4,
  useSettingValue: T5,

  enhancedSettingKeys: Set<string>,
  defaultSettings: Record<string, any>,
): [T1, T2, T3, T4, T5] {
  const useNextSettingKey = (key: string) => {
    const enableEnhancedSettings = useGeneralSettingKeyInternal("enhancedSettings")
    const settingValue = useSettingKey(key)
    const shouldBackToDefault = enhancedSettingKeys.has(key) && !enableEnhancedSettings
    if (!shouldBackToDefault) {
      return settingValue
    }

    return defaultSettings[key] === undefined ? settingValue : defaultSettings[key]
  }

  const useNextSettingSelector = (selector: (s: any) => any) => {
    const enableEnhancedSettings = useGeneralSettingKeyInternal("enhancedSettings")
    return useSettingSelector(
      useCallback(
        (settings) => {
          if (enableEnhancedSettings) {
            return selector(settings)
          }

          const enhancedSettings = { ...settings }
          for (const key of enhancedSettingKeys) {
            if (defaultSettings[key] !== undefined) {
              enhancedSettings[key] = defaultSettings[key]
            }
          }

          return selector(enhancedSettings)
        },
        [enableEnhancedSettings, selector],
      ),
    )
  }

  const useNextSettingKeys = (keys: string[]) => {
    const enableEnhancedSettings = useGeneralSettingKeyInternal("enhancedSettings")
    const rawSettingValues: string[] = useSettingKeys(keys)

    return useMemo(() => {
      if (enableEnhancedSettings) {
        return rawSettingValues
      }

      const result: string[] = []

      for (const [i, key] of keys.entries()) {
        if (enhancedSettingKeys.has(key) && defaultSettings[key] !== undefined) {
          result.push(defaultSettings[key])
        } else if (rawSettingValues[i] !== undefined) {
          result.push(rawSettingValues[i])
        }
      }

      return result
    }, [enableEnhancedSettings, keys, rawSettingValues])
  }

  const getNextSettings = () => {
    const settings = getSettings()
    const enableEnhancedSettings = jotaiStore.get(__generalSettingAtom).enhancedSettings

    if (enableEnhancedSettings) {
      return settings
    }

    const enhancedSettings = { ...settings }
    for (const key of enhancedSettingKeys) {
      if (defaultSettings[key] !== undefined) {
        enhancedSettings[key] = defaultSettings[key]
      }
    }

    return enhancedSettings
  }

  const useNextSettingValue = () => {
    const settingValues = useSettingValue()
    const enableEnhancedSettings = useGeneralSettingKeyInternal("enhancedSettings")

    return useMemo(() => {
      if (enableEnhancedSettings) {
        return settingValues
      }

      const result = { ...settingValues }
      for (const key of enhancedSettingKeys) {
        if (defaultSettings[key] !== undefined) {
          result[key] = defaultSettings[key]
        }
      }

      return result
    }, [enableEnhancedSettings, settingValues])
  }
  return [
    useNextSettingKey as T1,
    useNextSettingSelector as T2,
    useNextSettingKeys as T3,
    getNextSettings as T4,
    useNextSettingValue as T5,
  ]
}

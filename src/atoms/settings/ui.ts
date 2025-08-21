import { createSettingAtom } from "~/atoms/helper/setting"

import { jotaiStore } from "~/lib/jotai"
import { atom, useAtomValue } from "jotai"
import { useEventCallback } from "usehooks-ts"


import { hookEnhancedSettings } from "./general"

export const createDefaultUISettings = (): any => ({
  ...{},

  // Action Order
  toolbarOrder: 1,

  // Discover
  discoverLanguage:"zh-CN",
})


const {
  useSettingKey: useUISettingKeyInternal,
  useSettingSelector: useUISettingSelectorInternal,
  useSettingKeys: useUISettingKeysInternal,
  setSetting: setUISetting,
  clearSettings: clearUISettings,
  initializeDefaultSettings: initializeDefaultUISettings,
  getSettings: getUISettingsInternal,
  useSettingValue: useUISettingValueInternal,
  settingAtom: __uiSettingAtom,
} = createSettingAtom("ui", createDefaultUISettings)

const [useUISettingKey, useUISettingSelector, useUISettingKeys, getUISettings, useUISettingValue] =
  hookEnhancedSettings(
    useUISettingKeyInternal,
    useUISettingSelectorInternal,
    useUISettingKeysInternal,
    getUISettingsInternal,
    useUISettingValueInternal,
    new Set(), // TODO: enhancedUISettingKeys
    {}, // TODO: defaultUISettings
  )
export {
  __uiSettingAtom,
  clearUISettings,
  getUISettings,
  initializeDefaultUISettings,
  setUISetting,
  useUISettingKey,
  useUISettingKeys,
  useUISettingSelector,
  useUISettingValue,
}

export const uiServerSyncWhiteListKeys: (keyof any)[] = [
  "uiFontFamily",
  "readerFontFamily",
  "opaqueSidebar",
  // "customCSS",
]

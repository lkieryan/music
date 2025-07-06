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

const zenModeAtom = atom(false)

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

export const useIsZenMode = () => useAtomValue(zenModeAtom)
export const getIsZenMode = () => jotaiStore.get(zenModeAtom)

export const useSetZenMode = () => {
  return setZenMode
}
export const setZenMode = (checked: boolean) => {
  jotaiStore.set(zenModeAtom, checked)
}

export const useToggleZenMode = () => {
  const setZenMode = useSetZenMode()
  const isZenMode = useIsZenMode()
  return useEventCallback(() => {
    const newIsZenMode = !isZenMode
    document.documentElement.dataset.zenMode = newIsZenMode.toString()
    setZenMode(newIsZenMode)
  })
}

export const useRealInWideMode = () => {
  const wideMode = useUISettingKey("wideMode")
  const isZenMode = useIsZenMode()
  return wideMode || isZenMode
}

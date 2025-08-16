import type { JSX } from "react/jsx-runtime"

import type { SettingItem } from "./setting-builder"
import { createSettingBuilder } from "./setting-builder"

export const createDefineSettingItem =
  <T>(
    _getSetting: () => T,
    setSetting: (key: any, value: Partial<T>) => void,
    setPersisted?: (key: any, value: Partial<T>) => void,
  ) =>
  <K extends keyof T>(
    key: K,
    options: {
      label: string
      description?: string | JSX.Element
      onChange?: (value: T[K]) => void
      hide?: boolean
      persist?: boolean
    } & Omit<SettingItem<any>, "onChange" | "description" | "label" | "hide" | "key">,
  ): any => {
    const { label, description, onChange, hide, ...rest } = options

    return {
      key,
      label,
      description,
      onChange: (value: any) => {
        if (onChange) return onChange(value as any)
        const shouldPersist = Boolean(options.persist)
        if (shouldPersist && setPersisted) {
          setPersisted(key, value as any)
        } else {
          setSetting(key, value as any)
        }
      },
      disabled: hide,
      ...rest,
    } as SettingItem<any>
  }

export const createSetting = <T extends object>(
  useSetting: () => T,
  setSetting: (key: any, value: Partial<T>) => void,
  setPersisted?: (key: any, value: Partial<T>) => void,
) => {
  const SettingBuilder = createSettingBuilder(useSetting)
  const defineSettingItem = createDefineSettingItem(useSetting, setSetting, setPersisted)
  return {
    SettingBuilder,
    defineSettingItem,
  }
}

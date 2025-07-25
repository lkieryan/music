import { useMemo } from "react"

import { getMemoizedSettings } from "../settings-glob"
import type { SettingPageContext } from "../utils"

export const useSettingPageContext = (): SettingPageContext => {
  return {
    role: null,
    isInMASReview: false,
  }
}

export const useAvailableSettings = () => {
  const ctx = useSettingPageContext() 
  return useMemo(
    () => getMemoizedSettings().filter((t) => !t.loader.hideIf?.(ctx, {})),
    [ctx],
  )
}

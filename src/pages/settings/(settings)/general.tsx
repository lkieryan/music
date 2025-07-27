import { SettingGeneral } from "~/components/modules/settings/tabs/general"
import { SettingsTitle } from "~/components/modules/settings/title"
import { defineSettingPageData } from "~/components/modules/settings/utils"

const iconName = "i-mgc-settings-7-cute-re"
const priority = 1000

export const loader = defineSettingPageData({
  icon: iconName,
  name: "titles.general",
  priority,
})

export function Component() {
  return (
    <>
      <SettingsTitle />
      <SettingGeneral />
    </>
  )
}

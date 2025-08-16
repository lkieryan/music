import { SettingsTitle } from "~/components/modules/settings/title"
import { defineSettingPageData } from "~/components/modules/settings/utils"
import { SettingLyrics } from "~/components/modules/settings/tabs/lyrics"

const iconName = "i-mgc-music-3-cute-re"
const priority = 1100

export const loader = defineSettingPageData({
  icon: iconName,
  name: "titles.lyrics",
  priority,
})

export function Component() {
  return (
    <>
      <SettingsTitle />
      <SettingLyrics />
    </>
  )
}



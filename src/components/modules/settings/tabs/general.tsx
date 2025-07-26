
import {
  setGeneralSetting,
  useGeneralSettingKey,
  useGeneralSettingValue,
} from "~/atoms/settings/general"
import { createSetting } from "../helper/builder"

const { defineSettingItem: _defineSettingItem, SettingBuilder } = createSetting(
  useGeneralSettingValue,
  setGeneralSetting,
)


export const SettingGeneral = () => {
  return (
    <div className="mt-4">
      <SettingBuilder
        key={"reRenderKey".toString()}
        settings={[
          {
            type: "title",
            value: "Music",
          },
        ]}
      />
    </div>
  )
}
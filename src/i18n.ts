import app_cn from "@locales/app/zh-CN.json"
import common_cn from "@locales/common/zh-CN.json"
import settings_cn from "@locales/settings/zh-CN.json"
import app_en from "@locales/app/en-US.json"
import common_en from "@locales/common/en-US.json"
import settings_en from "@locales/settings/en-US.json"
import { initReactI18next } from "react-i18next";
import i18next from "i18next"
import { atom } from "jotai"
import { jotaiStore } from "./lib/jotai"

export const i18nAtom = atom(i18next)

export const ns = ["common", "app", "settings"] as const

export const fallbackLanguage = "zh-CN"

export const initI18n = async () => {
  const i18next = jotaiStore.get(i18nAtom)

  const resources = {
	en: {
	  app: app_en,
	  common: common_en,
	  settings: settings_en,
    },
	"zh-CN": {
	  app: app_cn,
      common: common_cn,
	  settings: settings_cn,
	},
  }


  await i18next.use(initReactI18next).init({
    ns,
    lng: fallbackLanguage,
    fallbackLng: {
      default: [fallbackLanguage],
      "zh-TW": ["zh-CN", fallbackLanguage],
    },
    defaultNS: "common",
    debug: import.meta.env.DEV,
    resources,
  })
}

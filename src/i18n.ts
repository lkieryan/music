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

const langs = ["en", "zh-CN"] as const
export const currentSupportedLanguages = langs as readonly string[]
export type RendererSupportedLanguages = (typeof langs)[number]

export const i18nAtom = atom(i18next)

export const ns = ["common", "app", "settings"] as const

export const fallbackLanguage = "zh-CN"

export const defaultResources = {
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
} satisfies Record<
RendererSupportedLanguages,
Partial<Record<(typeof ns)[number], Record<string, string>>>
>

export const initI18n = async () => {
  const i18nInst = jotaiStore.get(i18nAtom)

  // Try to load active language from backend settings
  let initialLang = fallbackLanguage
  try {
    const { loadSettings } = await import('~/services/settings')
    const prefLang = await loadSettings<string>('general.language')
    if (prefLang && typeof prefLang === 'string') {
      initialLang = prefLang
    }
  } catch {
    // ignore, fallback to default
  }

  await i18nInst.use(initReactI18next).init({
    ns,
    lng: initialLang,
    fallbackLng: {
      default: [fallbackLanguage],
      'zh-TW': ['zh-CN', fallbackLanguage],
    },
    defaultNS: 'common',
    debug: import.meta.env.DEV,
    resources: defaultResources,
  })

  // Listen for language changes from settings
  try {
    const { listenGeneral } = await import('~/atoms/settings/general')
    listenGeneral((key, value) => {
      if (key === 'language') {
        changeAppLanguage(value || fallbackLanguage)
      }
    })
  } catch (e) {
    console.warn('[i18n] Failed to setup language listener:', e)
  }
}

export async function changeAppLanguage(lang: RendererSupportedLanguages) {
  const i18nInst = jotaiStore.get(i18nAtom)
  await i18nInst.changeLanguage(lang)
}

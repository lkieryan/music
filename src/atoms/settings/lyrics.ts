import { atomWithStorage } from "jotai/utils"
import { createSettingAtom } from "~/atoms/helper/setting"
import type { LyricsSettings } from "~/types/bindings"
import type { DomainBinding } from "~/services/settings"
import { createDomainBinding } from "~/services/settings"

// ==================================================================
//                     歌词相关简单原子状态
// ==================================================================

/**
 * 是否启用提前歌词行时序的功能。
 * 即将原歌词行的初始时间时序提前，以便在歌词滚动结束后刚好开始播放（逐词）歌词效果。
 */
export const advanceLyricDynamicLyricTimeAtom = atomWithStorage(
	"amll-player.advanceLyricDynamicLyricTimeAtom",
	false,
);

// ==================================================================
//                     完整歌词设置管理
// ==================================================================

type LyricsDefaults = Strictify<LyricsSettings>

export const createDefaultLyricsSettings = (): LyricsDefaults => ({
  // Appearance / implementation
  playerImplementation: "dom",
  fontFamily: "",
  fontWeight: "",
  letterSpacing: "normal",
  sizePreset: "medium",
  lineBlurEffect: true,
  lineScaleEffect: true,
  lineSpringAnimation: true,
  advanceLineTiming: false,
  wordFadeWidth: 0.5,

  // Content toggles
  translationLine: true,
  romanLine: true,
  swapTransRomanLine: false,

})

const {
  useSettingKey: useLyricsSettingKey,
  useSettingSelector: useLyricsSettingSelector,
  useSettingKeys: useLyricsSettingKeys,
  setSetting: setLyricsSetting,
  clearSettings: clearLyricsSettings,
  initializeDefaultSettings: initializeDefaultLyricsSettings,
  getSettings: getLyricsSettings,
  useSettingValue: useLyricsSettingValue,
  settingAtom: __lyricsSettingAtom,
} = createSettingAtom("lyrics", createDefaultLyricsSettings)

export {
  __lyricsSettingAtom,
  useLyricsSettingKey,
  useLyricsSettingSelector,
  useLyricsSettingKeys,
  setLyricsSetting,
  clearLyricsSettings,
  initializeDefaultLyricsSettings,
  getLyricsSettings,
  useLyricsSettingValue,
}

let __lyricsBinding: DomainBinding<LyricsDefaults> | null = null

export function getLyricsBinding() {
  if (!__lyricsBinding) {
    __lyricsBinding = createDomainBinding<LyricsDefaults, LyricsSettings>({
      domain: "lyrics",
      defaultFactory: createDefaultLyricsSettings,
      setLocal: (k, v) => setLyricsSetting(k as any, v as any),
      keyMap: {} as any,
    })
  }
  return __lyricsBinding
}

export async function hydrateLyrics() {
  await getLyricsBinding().hydrate()
}

export function listenLyrics(onAfterChange?: (k: keyof LyricsDefaults, v: any) => void) {
  return getLyricsBinding().listen(onAfterChange as any)
}

export function setLyrics<K extends keyof LyricsDefaults>(key: K, value: LyricsDefaults[K]) {
  return getLyricsBinding().set(key, value)
}

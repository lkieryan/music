import { createSettingAtom } from "~/atoms/helper/setting"
import type { DomainBinding } from "~/services/settings"
import { createDomainBinding } from "~/services/settings"
import type { MusicSettings } from "~/types/bindings"

type MusicDefaults = Strictify<MusicSettings>

export const createDefaultMusicSettings = (): MusicDefaults => ({
  // Platform/source selection
  source: {
    mode: "all",
    ids: [],
  },
  // User-preferred order for non-Home sources (plugin IDs)
  sourcesOrder: [],
  // Playback preferences
  playback: {
    normalize: false,
    crossfadeMs: 0,
    gapless: true,
  },
  // Audio effects chain configuration
  effects: {
    enabled: false,
    chain: [],
  },
})

const {
  useSettingKey: useMusicSettingKey,
  useSettingSelector: useMusicSettingSelector,
  useSettingKeys: useMusicSettingKeys,
  setSetting: setMusicSetting,
  clearSettings: clearMusicSettings,
  initializeDefaultSettings: initializeDefaultMusicSettings,
  getSettings: getMusicSettings,
  useSettingValue: useMusicSettingValue,
  settingAtom: __musicSettingAtom,
} = createSettingAtom("music", createDefaultMusicSettings)

export {
  __musicSettingAtom,
  useMusicSettingKey,
  useMusicSettingSelector,
  useMusicSettingKeys,
  setMusicSetting,
  clearMusicSettings,
  initializeDefaultMusicSettings,
  getMusicSettings,
  useMusicSettingValue,
}

let __musicBinding: DomainBinding<MusicDefaults> | null = null

export function getMusicBinding() {
  if (!__musicBinding) {
    __musicBinding = createDomainBinding<MusicDefaults, MusicSettings>({
      domain: "music",
      defaultFactory: createDefaultMusicSettings,
      setLocal: (k, v) => setMusicSetting(k as any, v as any),
      keyMap: {} as any,
    })
  }
  return __musicBinding
}

export async function hydrateMusic() {
  await getMusicBinding().hydrate()
}

export function listenMusic(onAfterChange?: (k: keyof MusicDefaults, v: any) => void) {
  return getMusicBinding().listen(onAfterChange as any)
}

export function setMusic<K extends keyof MusicDefaults>(key: K, value: MusicDefaults[K]) {
  return getMusicBinding().set(key, value)
}


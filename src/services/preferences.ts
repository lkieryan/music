import { invoke } from '~/lib/tauri'

export async function loadPreference<T = any>(key: string): Promise<T | undefined> {
  return invoke('load_selective', { key }).catch(() => undefined)
}

export async function savePreference<T = any>(key: string, value: T): Promise<void> {
  await invoke('save_selective', { key, value })
}

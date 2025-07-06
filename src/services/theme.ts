import { invoke } from '~/lib/tauri'
import type { ThemeDetails } from '~/types/theme'

export const ThemeCommands = {
  getState: 'get_theme_handler_state',
  save: 'save_theme',
  remove: 'remove_theme',
  load: 'load_theme',
  loadAll: 'load_all_themes',
  css: 'get_css',
} as const

export async function loadAllThemes(): Promise<Record<string, ThemeDetails>> {
  return invoke(ThemeCommands.loadAll)
}

export async function loadTheme(id: string): Promise<ThemeDetails> {
  return invoke(ThemeCommands.load, { id })
}

export async function getThemeCss(id: string): Promise<string> {
  return invoke(ThemeCommands.css, { id })
}

export async function saveTheme(theme: ThemeDetails) {
  return invoke(ThemeCommands.save, { theme })
}

export async function removeTheme(id: string) {
  return invoke(ThemeCommands.remove, { id })
}

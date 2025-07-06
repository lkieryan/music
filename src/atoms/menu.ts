import { atom } from 'jotai'

export type SidebarPosition = 'left' | 'right'
export type ToolbarDensity = 'regular' | 'compact'

export type MenuItem = {
  id: string
  label: string
  icon?: string
  path?: string
  fixed: boolean
  closable?: boolean
  badge?: number
  order?: number
  accent?: string // container/accent color for side indicator
}

// UI atoms
export const sidebarPositionAtom = atom<SidebarPosition>('left')
export const toolbarDensityAtom = atom<ToolbarDensity>('regular')

// Data atoms
export const fixedMenusAtom = atom<MenuItem[]>([
  { id: 'home', label: 'Home', icon: '/vite.svg', fixed: true, path: '/', accent: '#667eea' },
  { id: 'library', label: 'Library', icon: '/vite.svg', fixed: true, path: '/', accent: '#22c55e' },
  { id: 'settings', label: 'Settings', icon: '/vite.svg', fixed: true, path: '/', accent: '#f59e0b' },
  { id: 'about', label: 'About', icon: '/vite.svg', fixed: true, path: '/', accent: '#ec4899' },
])

export const dynamicMenusAtom = atom<MenuItem[]>([
  { id: 'recent-1', label: 'Recent One', icon: '/vite.svg', fixed: false, closable: true, path: '/', accent: '#60a5fa' },
  { id: 'recent-2', label: 'Recent Two', icon: '/vite.svg', fixed: false, closable: true, path: '/', accent: '#34d399' },
  { id: 'recent-3', label: 'Recent Three', icon: '/vite.svg', fixed: false, closable: true, path: '/', accent: '#f472b6' },
  { id: 'recent-4', label: 'Recent Four', icon: '/vite.svg', fixed: false, closable: true, path: '/', accent: '#fbbf24' },
  { id: 'recent-5', label: 'Recent Five', icon: '/vite.svg', fixed: false, closable: true, path: '/', accent: '#a78bfa' },
  { id: 'recent-6', label: 'Recent Six', icon: '/vite.svg', fixed: false, closable: true, path: '/', accent: '#f87171' },
])

export const activeMenuIdAtom = atom<string>('home')

// Actions
export const addDynamicMenuAtom = atom(null, (get, set, item: MenuItem) => {
  const list = get(dynamicMenusAtom)
  if (list.some((m) => m.id === item.id)) return
  set(dynamicMenusAtom, [...list, { ...item, fixed: false, closable: true }])
})

export const closeDynamicMenuAtom = atom(null, (get, set, id: string) => {
  const list = get(dynamicMenusAtom)
  set(dynamicMenusAtom, list.filter((m) => m.id !== id))
  const active = get(activeMenuIdAtom)
  if (active === id) {
    // fallback: first fixed, else first dynamic
    const fixed = get(fixedMenusAtom)
    const next = fixed[0]?.id ?? get(dynamicMenusAtom)[0]?.id ?? 'home'
    set(activeMenuIdAtom, next)
  }
})

export const setActiveMenuAtom = atom(null, (_get, set, id: string) => {
  set(activeMenuIdAtom, id)
})


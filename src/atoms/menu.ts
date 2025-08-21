import { atom } from 'jotai'
import type { FC, SVGProps } from 'react'
import HomeIcon from '~/assets/icons/home.svg?react'
import LibraryIcon from '~/assets/icons/library.svg?react'

export type SidebarPosition = 'left' | 'right'
export type ToolbarDensity = 'regular' | 'compact'
export type MenuSortMode = 'label-asc' | 'label-desc' | 'custom'

export type MenuItem = {
  id: string
  label: string
  // icon can be:
  // - an absolute public path (starts with "/")
  // - a filename under src/assets/icons (e.g., "home.svg")
  // - a React component (SVGR) for maximum flexibility
  icon?: string | FC<SVGProps<SVGSVGElement>>
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
export const menuSortAtom = atom<MenuSortMode>('label-asc')

// Data atoms
export const menusAtom = atom<MenuItem[]>([
  // labels use i18n keys under namespace "app"; icons as React components
  { id: 'home', label: 'menu.home', icon: HomeIcon, fixed: true, path: '/', accent: '#667eea', order: 1 },
  { id: 'local', label: 'menu.local_media', icon: LibraryIcon, fixed: true, path: '/local', accent: '#22c55e', order: 2 },
])

export const activeMenuIdAtom = atom<string>('home')


export const setActiveMenuAtom = atom(null, (_get, set, id: string) => {
  set(activeMenuIdAtom, id)
})


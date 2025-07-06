import { atom } from 'jotai'
import { atomWithStorage } from 'jotai/utils'

export type ToolbarMode = 'single' | 'multi' | 'compact'
export type SidebarPosition = 'left' | 'right'

// Persisted layout state (can be changed from anywhere)
export const toolbarModeAtom = atomWithStorage<ToolbarMode>('layout.toolbarMode', 'multi')
export const sidebarPositionAtom = atomWithStorage<SidebarPosition>('layout.sidebarPosition', 'left')

// Derived compact mode: true when toolbar mode is 'compact'
export const compactModeAtom = atom((get) => get(toolbarModeAtom) === 'compact')

// Sidebar width (px), persisted; used when not in compact mode
export const sidebarWidthAtom = atomWithStorage<number>('layout.sidebarWidth', 280)
export const sidebarMinWidth = 160
export const sidebarMaxWidth = 520

// Player layout
export type PlayerPlacement = 'none' | 'global-bottom' | 'content-bottom' | 'sidebar'
export const playerPlacementAtom = atomWithStorage<PlayerPlacement>('layout.playerPlacement', 'none')
export const playerHeightAtom = atomWithStorage<number>('layout.playerHeight', 64)
export const playerVisibleAtom = atomWithStorage<boolean>('layout.playerVisible', true)


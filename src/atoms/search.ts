import { atom } from 'jotai'
import type { SearchResult } from '~/types/sdk-search'
import type { MusicSelection } from '~/services/music-api'

// Search input value (temporary state, not persisted)
export const searchTermAtom = atom<string>('')

// Current search loading state
export type SearchLoadingState = 'idle' | 'loading' | 'success' | 'error'
export const searchLoadingAtom = atom<SearchLoadingState>('idle')

// Search results (temporary state, not persisted)
export const searchResultAtom = atom<SearchResult | null>(null)

// Search error message
export const searchErrorAtom = atom<string | null>(null)

// Current search provider selection (temporary state)
export const searchSelectorAtom = atom<MusicSelection | null>(null)

// Last search term (for displaying on results page)
export const lastSearchTermAtom = atom<string>('')

// Helper action atom to clear search state
export const clearSearchAtom = atom(
  null,
  (_get, set) => {
    set(searchResultAtom, null)
    set(searchErrorAtom, null)
    set(searchLoadingAtom, 'idle')
    set(lastSearchTermAtom, '')
  }
)

import { create } from 'zustand'

interface Song {
  id: string
  title: string
  artist: string
  album: string
  duration: number
  cover?: string
  url: string
}

interface PlayerState {
  isPlaying: boolean
  currentSong: Song | null
  position: number
  duration: number
  volume: number
  playlist: Song[]
  
  setIsPlaying: (isPlaying: boolean) => void
  setCurrentSong: (song: Song | null) => void
  setPosition: (position: number) => void
  setDuration: (duration: number) => void
  setVolume: (volume: number) => void
  setPlaylist: (playlist: Song[]) => void
  addToPlaylist: (song: Song) => void
  removeFromPlaylist: (songId: string) => void
}

const usePlayerStore = create<PlayerState>((set) => ({
  isPlaying: false,
  currentSong: null,
  position: 0,
  duration: 0,
  volume: 1,
  playlist: [],
  
  setIsPlaying: (isPlaying) => set({ isPlaying }),
  setCurrentSong: (song) => set({ currentSong: song }),
  setPosition: (position) => set({ position }),
  setDuration: (duration) => set({ duration }),
  setVolume: (volume) => set({ volume }),
  
  setPlaylist: (playlist) => set({ playlist }),
  
  addToPlaylist: (song) =>
    set((state) => ({
      playlist: [...state.playlist, song],
    })),
    
  removeFromPlaylist: (songId) =>
    set((state) => ({
      playlist: state.playlist.filter((song) => song.id !== songId),
    })),
}))

export default usePlayerStore
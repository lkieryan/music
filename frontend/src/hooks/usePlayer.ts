import { useCallback } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import usePlayerStore from '@stores/playerStore'

export default function usePlayer() {
  const {
    isPlaying,
    currentSong,
    position,
    duration,
    volume,
    setIsPlaying,
    setCurrentSong,
    setPosition,
    setDuration,
    setVolume,
  } = usePlayerStore()

  const play = useCallback(async (songId: string) => {
    try {
      await invoke('play_song', { songId })
      setIsPlaying(true)
    } catch (error) {
      console.error('Failed to play song:', error)
    }
  }, [setIsPlaying])

  const pause = useCallback(async () => {
    try {
      await invoke('pause_playback')
      setIsPlaying(false)
    } catch (error) {
      console.error('Failed to pause playback:', error)
    }
  }, [setIsPlaying])

  const resume = useCallback(async () => {
    try {
      await invoke('resume_playback')
      setIsPlaying(true)
    } catch (error) {
      console.error('Failed to resume playback:', error)
    }
  }, [setIsPlaying])

  const stop = useCallback(async () => {
    try {
      await invoke('stop_playback')
      setIsPlaying(false)
      setPosition(0)
    } catch (error) {
      console.error('Failed to stop playback:', error)
    }
  }, [setIsPlaying, setPosition])

  const seek = useCallback(async (position: number) => {
    try {
      await invoke('seek_to_position', { position })
      setPosition(position)
    } catch (error) {
      console.error('Failed to seek:', error)
    }
  }, [setPosition])

  const setPlayerVolume = useCallback(async (volume: number) => {
    try {
      await invoke('set_volume', { volume })
      setVolume(volume)
    } catch (error) {
      console.error('Failed to set volume:', error)
    }
  }, [setVolume])

  const previous = useCallback(async () => {
    try {
      await invoke('previous_song')
    } catch (error) {
      console.error('Failed to play previous song:', error)
    }
  }, [])

  const next = useCallback(async () => {
    try {
      await invoke('next_song')
    } catch (error) {
      console.error('Failed to play next song:', error)
    }
  }, [])

  return {
    // 状态
    isPlaying,
    currentSong,
    position,
    duration,
    volume,
    
    // 控制方法
    play,
    pause,
    resume,
    stop,
    seek,
    setVolume: setPlayerVolume,
    previous,
    next,
  }
}
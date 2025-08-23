import { convertFileSrc } from '@tauri-apps/api/core'

/**
 * Resolve a provided path or URL to a browser-usable image URL.
 * - If it's an http/https/data/blob URL, returns as-is.
 * - If it's a local file path, converts to Tauri asset URL via convertFileSrc.
 * - Returns null when cannot resolve.
 */
export function resolveImageUrl(input?: string | null): string | null {
  if (!input) return null
  const s = input.trim()
  if (!s) return null

  // Already a usable URL
  if (s.startsWith('data:') || s.startsWith('blob:') || /^https?:\/\//i.test(s)) {
    return s
  }

  // Try to convert local file path
  try {
    return convertFileSrc(s)
  } catch (err) {
    console.warn('[resolveImageUrl] Failed to convert local path:', s, err)
    return null
  }
}

/**
 * Try to resolve song cover by priority: song-high -> album-high -> song-low -> album-low
 */
export function resolveSongCoverUrl(song: any): string | null {
  if (!song) return null
  const candidates: Array<string | undefined | null> = [
    song.song_coverPath_high,
    song.album?.album_coverPath_high,
    song.song_coverPath_low,
    song.album?.album_coverPath_low,
    // Optional generic fields for robustness
    song.coverPath,
    song.album?.coverPath,
    song.cover_url,
  ]

  for (const c of candidates) {
    const url = resolveImageUrl(c ?? null)
    if (url) return url
  }
  return null
}

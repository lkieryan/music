import { convertFileSrc } from '@tauri-apps/api/core'
import type { MediaContent } from '~/types/bindings'

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

  // Handle protocol-relative URLs (e.g., //domain/path)
  // Treat them as HTTPS by default
  if (s.startsWith('//')) {
    return `https:${s}`
  }

  // Already a usable URL
  if (
    s.startsWith('data:') ||
    s.startsWith('blob:') ||
    /^https?:\/\//i.test(s) ||
    s.startsWith('asset://')
  ) {
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
 * Try to resolve track cover by priority: track-high -> album-high -> track-low -> album-low
 */
export function resolveTrackCoverUrl(track: MediaContent): string | null {
  if (!track) return null
  const candidates: Array<string | undefined | null> = [
    track.track_coverpath_high,
    track.album?.album_coverPath_high,
    track.track_coverpath_low,
    track.album?.album_coverPath_low,
  ]

  for (const c of candidates) {
    const url = resolveImageUrl(c ?? null)
    if (url) return url
  }
  return null
}

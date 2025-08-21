import type { FC } from "react"
import { useEffect } from "react"
import { BackgroundRender } from "@applemusic-like-lyrics/react"
import { useAtomValue } from "jotai"
import { useBackgroundSettingsValue } from "~/atoms/settings/themes"
import { musicCoverAtom, musicCoverIsVideoAtom, lowFreqVolumeAtom } from "~/atoms/player/data-atoms"
import { lyricBackgroundRendererAtom } from "~/atoms/player/config-atoms"
import { jotaiStore } from "~/lib/jotai"
import { appThemeAtom } from "~/atoms/settings/themes"

/**
 * DynamicCoverBackground
 * Fullscreen background that renders from current album cover or CSS value.
 * - When renderer = "css-bg": render a plain CSS background layer
 * - Otherwise: render BackgroundRender with album cover, fps, scale, etc.
 */
const DynamicCoverBackground: FC = () => {
  const bg = useBackgroundSettingsValue()
  const album = useAtomValue(musicCoverAtom)
  const albumIsVideo = useAtomValue(musicCoverIsVideoAtom)
  const lowFreqVolume = useAtomValue(lowFreqVolumeAtom)

  // Force dark theme while dynamic cover is active; restore previous theme on unmount
  useEffect(() => {
    const prev = jotaiStore.get(appThemeAtom)
    if (prev !== 'dark') {
      jotaiStore.set(appThemeAtom, 'dark')
    }
    return () => {
      if (prev !== 'dark') {
        jotaiStore.set(appThemeAtom, prev)
      }
    }
  }, [])

  const isCss = bg.backgroundRenderer === "css-bg"
  const cssBg = bg.cssBackgroundProperty || "#111111"
  const legacyRenderer = useAtomValue(lyricBackgroundRendererAtom)

  if (isCss) {
    return (
      <div
        className="fixed inset-0 pointer-events-none z-0"
        style={{
          background: cssBg,
        }}
      />
    )
  }

  // If legacyRenderer.renderer is a string (unexpected here), fall back to CSS layer to avoid constructor error
  if (typeof legacyRenderer.renderer === 'string') {
    return (
      <div
        className="fixed inset-0 pointer-events-none z-0"
        style={{ background: legacyRenderer.renderer as string }}
      />
    )
  }

  return (
    <BackgroundRender
      className="fixed inset-0 pointer-events-none z-0"
      album={album}
      albumIsVideo={albumIsVideo}
      lowFreqVolume={lowFreqVolume}
      renderScale={bg.backgroundRenderScale ?? 1}
      fps={bg.backgroundFps ?? 60}
      renderer={legacyRenderer.renderer}
      staticMode={!!bg.backgroundStaticMode}
      style={{
        minWidth: 0,
        minHeight: 0,
        overflow: "hidden",
      }}
    />
  )
}

export default DynamicCoverBackground

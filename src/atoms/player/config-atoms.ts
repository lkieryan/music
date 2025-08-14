import { atom } from "jotai"
import { atomWithStorage } from "jotai/utils"
import type { BackgroundRenderProps } from "@applemusic-like-lyrics/react"

export enum PlayerControlsType {
  Controls = "controls",
  FFT = "fft",
  None = "none",
}

export enum VerticalCoverLayout {
  Auto = "auto",
  ForceNormal = "force-normal",
  ForceImmersive = "force-immersive",
}

export const showMusicNameAtom = atomWithStorage("ui.showMusicName", true)
export const showMusicArtistsAtom = atomWithStorage("ui.showMusicArtists", true)
export const showMusicAlbumAtom = atomWithStorage("ui.showMusicAlbum", false)
export const showRemainingTimeAtom = atomWithStorage("ui.showRemainingTime", true)
export const showVolumeControlAtom = atomWithStorage("ui.showVolumeControl", true)
export const playerControlsTypeAtom = atomWithStorage<PlayerControlsType>(
  "ui.playerControlsType",
  PlayerControlsType.Controls,
)
export const verticalCoverLayoutAtom = atomWithStorage<VerticalCoverLayout>(
  "ui.verticalCoverLayout",
  VerticalCoverLayout.Auto,
)

export const lyricBackgroundRendererAtom = atom<{ renderer?: BackgroundRenderProps["renderer"] | string }>({ renderer: undefined })
export const showBottomControlAtom = atomWithStorage("ui.showBottomControl", true)

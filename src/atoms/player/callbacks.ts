import { atom } from "jotai"
import type { LyricLineMouseEvent } from "@applemusic-like-lyrics/core"
import type { LyricPlayerRef } from "@applemusic-like-lyrics/react"

export interface Callback<Args extends unknown[], Result = void> {
  onEmit?: (...args: Args) => Result
}

const c = <Args extends unknown[], Result = void>(
  _onEmit: (...args: Args) => Result,
): Callback<Args, Result> => ({} as any)

// ==================================================================
//                        播放器控制回调
// ==================================================================


/**
 * 点击音质标签时的回调。
 */
export const onClickAudioQualityTagAtom = atom(c(() => {}))

/**
 * 请求打开菜单时的回调。
 */
export const onRequestOpenMenuAtom = atom(c(() => {}))

/**
 * 播放或暂停音乐时的回调。
 */
export const onPlayOrResumeAtom = atom(c(() => {}))

/**
 * 请求播放上一首歌曲时的回调。
 */
export const onRequestPrevTrackAtom = atom(c(() => {}))

/**
 * 请求播放下一首歌曲时的回调。
 */
export const onRequestNextTrackAtom = atom(c(() => {}))

/**
 * 拖拽进度条或点击进度条时的回调。
 */
export const onSeekPositionAtom = atom(c((_position: number) => {}))

/**
 * 点击歌词行时的回调。
 */
export const onLyricLineClickAtom = atom(
  c((_evt: LyricLineMouseEvent, _playerRef: LyricPlayerRef | null) => {}),
)

/**
 * 右键歌词行时的回调。
 */
export const onLyricLineContextMenuAtom = atom(
  c((_evt: LyricLineMouseEvent, _playerRef: LyricPlayerRef | null) => {}),
)

/**
 * 调整音量时的回调。
 */
export const onChangeVolumeAtom = atom(c((_volume: number) => {}))

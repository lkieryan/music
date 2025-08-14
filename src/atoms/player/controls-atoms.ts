import { atom, type Atom } from "jotai"
import { musicPlayingPositionAtom } from "./data-atoms"

export enum RepeatMode {
  Off = "off",
  One = "one",
  All = "all",
}

export const isShuffleActiveAtom = atom<boolean>(false)
export const repeatModeAtom = atom<RepeatMode>(RepeatMode.Off)

export const toggleShuffleActionAtom = atom(null, (get, set) => {
  set(isShuffleActiveAtom, !get(isShuffleActiveAtom))
})

export const cycleRepeatModeActionAtom = atom(null, (get, set) => {
  const curr = get(repeatModeAtom)
  const next = curr === RepeatMode.Off ? RepeatMode.All : curr === RepeatMode.All ? RepeatMode.One : RepeatMode.Off
  set(repeatModeAtom, next)
})

export const positionSourceAtom = atom<Atom<number>>(musicPlayingPositionAtom)
export const correctedMusicPlayingPositionAtom = atom((get) => get(get(positionSourceAtom)))

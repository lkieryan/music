import { useAtomValue } from 'jotai'
import { type FC, useLayoutEffect } from 'react'
import { cn } from '~/lib/helper'
import { isLyricPageOpenedAtom } from '~/atoms/player'
import { PrebuiltLyricPlayer } from './lyirc'
import styles from './lyric-page.module.css'

export const LyricPage: FC = () => {
  const isLyricPageOpened = useAtomValue(isLyricPageOpenedAtom)

  useLayoutEffect(() => {
    console.log('isLyricPageOpened', isLyricPageOpened)
    if (isLyricPageOpened) {
      document.body.dataset.amllLyricsOpen = ""
    } else {
      delete document.body.dataset.amllLyricsOpen
    }
  }, [isLyricPageOpened])

  return (
    <PrebuiltLyricPlayer
      id="amll-lyric-player"
      className={cn(
        styles.lyricPage,
        isLyricPageOpened && styles.opened
      )}
    />
  )
}
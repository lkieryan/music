import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { type FC, useLayoutEffect, useRef } from 'react'
import { cn } from '~/lib/helper'
import { resolveImageUrl } from '~/lib/image'

import { 
  musicNameAtom,
  musicArtistsAtom, 
  musicCoverAtom,
  musicPlayingAtom,
  onPlayOrResumeAtom,
  onRequestPrevSongAtom,
  onRequestNextSongAtom,
  playlistCardOpenedAtom,
  isLyricPageOpenedAtom
} from '~/atoms/player'

import { playerVisibleAtom } from '~/atoms/layout'
import styles from './global.module.css'

// 导入你现有的 SVG 图标
import IconPlay from '~/assets/icons/icon_play.svg?react'
import IconPause from '~/assets/icons/icon_pause.svg?react'
import IconForward from '~/assets/icons/icon_forward.svg?react'
import IconRewind from '~/assets/icons/icon_rewind.svg?react'
import IconLyrics from '~/assets/icons/icon_lyrics.svg?react'
import MenuIcon from '~/assets/icons/menu.svg?react'

// 导入 AMLL 组件，如果不可用则使用本地实现
import { 
  TextMarquee,
  MediaButton
} from '@applemusic-like-lyrics/react-full'

// 播放列表卡片占位组件
const NowPlaylistCard: FC<{ className?: string }> = ({ className }) => {
  return (
    <div className={cn(styles.playlistCard, className)}>
      <div className="p-4">
        <div className="text-white text-sm font-medium mb-2">播放列表</div>
        <div className="text-gray-300 text-xs">暂无播放列表</div>
      </div>
    </div>
  )
}

export function GlobalPlayer({ height }: { height: number }) {
  const playerVisible = useAtomValue(playerVisibleAtom)
  const musicName = useAtomValue(musicNameAtom)
  const musicArtists = useAtomValue(musicArtistsAtom)
  const musicPlaying = useAtomValue(musicPlayingAtom)
  const musicCover = useAtomValue(musicCoverAtom)
  const [playlistOpened, setPlaylistOpened] = useAtom(playlistCardOpenedAtom)
  const setLyricPageOpened = useSetAtom(isLyricPageOpenedAtom)

  // Use callback atoms' value and call onEmit
  const onPlayOrResume = useAtomValue(onPlayOrResumeAtom)
  const onRequestPrevSong = useAtomValue(onRequestPrevSongAtom)
  const onRequestNextSong = useAtomValue(onRequestNextSongAtom)

  const playbarRef = useRef<HTMLDivElement>(null)

  useLayoutEffect(() => {
    const playbarEl = playbarRef.current
    if (!playbarEl) return
    
    const updateSafeBound = () => {
      const { top } = playbarEl.getBoundingClientRect()
      document.body.style.setProperty(
        '--amll-player-playbar-bottom',
        `${window.innerHeight - top}px`
      )
    }
    
    const observer = new ResizeObserver(updateSafeBound)
    window.addEventListener('resize', updateSafeBound)
    observer.observe(playbarEl)
    updateSafeBound()
    
    return () => {
      window.removeEventListener('resize', updateSafeBound)
      observer.disconnect()
    }
  }, [])

  if (!playerVisible) {
    return null
  }

  return (
    <>
      {/* 播放列表卡片 */}
      {playlistOpened && (
        <div className="absolute right-3 z-10" 
             style={{ bottom: 'calc(var(--amll-player-playbar-bottom, 80px) + 12px)' }}>
          <NowPlaylistCard className={styles.playlistCard} />
        </div>
      )}
      
      {/* 主播放栏 */}
      <div 
        className={cn(
          "w-full bg-transparent backdrop-blur-md backdrop-saturate-[120%] border-black/8 flex-shrink-0 overflow-hidden",
          styles.playBar,
          !playerVisible && styles.hide
        )}
        style={{ height }}
        ref={playbarRef}
      >
        <div className="flex items-center justify-between w-full">
          {/* 左侧：封面和歌曲信息 */}
          <div className="flex items-center flex-1 min-w-0 basis-1/3">
            <button
              className={cn(styles.coverButton, 'press-anim-parent')}
              type="button"
              style={{
                // Resolve cover URL for local or network paths
                backgroundImage: resolveImageUrl(musicCover) ? `url(${resolveImageUrl(musicCover)})` : 'none',
                backgroundColor: resolveImageUrl(musicCover) ? 'transparent' : '#0000001a'
              }}
              onClick={() => setLyricPageOpened(true)}
            >
              <div className={styles.lyricIconButton}>
                <IconLyrics width={28} height={28} className="press-anim" />
              </div>
            </button>
            
            <div className="flex flex-col justify-center ml-3 flex-1 min-w-0 overflow-hidden">
              <TextMarquee>{musicName || '未播放'}</TextMarquee>
              <TextMarquee>
                {musicArtists.length > 0 
                  ? musicArtists.map(v => v.name).join(', ')
                  : '未知艺术家'
                }
              </TextMarquee>
            </div>
          </div>

          {/* 中间：播放控制按钮（桌面端显示） */}
          <div className="hidden sm:flex items-center justify-center flex-1 basis-1/3 gap-8">
            <MediaButton 
              className="press-anim-parent cursor-pointer !min-w-12 !min-h-12 !p-2"
              onClick={() => onRequestPrevSong.onEmit?.()} 
              style={{ scale: "1.5" }}
            >
              <IconRewind style={{ scale: "1.25" }} className="press-anim" />
            </MediaButton>
            
            <MediaButton 
              className="press-anim-parent cursor-pointer !min-w-12 !min-h-12 !p-2"
              onClick={() => onPlayOrResume.onEmit?.()} 
              style={{ scale: "1.5" }}
            >
              {musicPlaying ? (
                <IconPause style={{ scale: "0.75" }} className="press-anim" />
              ) : (
                <IconPlay style={{ scale: "0.75" }} className="press-anim" />
              )}
            </MediaButton>
            
            <MediaButton 
              className="press-anim-parent cursor-pointer !min-w-12 !min-h-12 !p-2"
              onClick={() => onRequestNextSong.onEmit?.()} 
              style={{ scale: "1.5" }}
            >
              <IconForward style={{ scale: "1.25" }} className="press-anim" />
            </MediaButton>
          </div>

          {/* 右侧：控制按钮 */}
          <div className="flex items-center justify-end flex-1 basis-1/3 gap-1">
            {/* 移动端播放控制 */}
            <div className="flex sm:hidden items-center gap-1">
              <button 
                className={cn('press-anim-parent', styles.mediaButton, "!min-w-10 !min-h-10 !p-2")}
                onClick={() => onRequestPrevSong.onEmit?.()}
              >
                <IconRewind className="w-4 h-4 press-anim" />
              </button>
              <button 
                className={cn('press-anim-parent', styles.mediaButton, "!min-w-10 !min-h-10 !p-2")}
                onClick={() => onPlayOrResume.onEmit?.()}
              >
                {musicPlaying ? <IconPause className="w-4 h-4 press-anim" /> : <IconPlay className="w-4 h-4 press-anim" />}
              </button>
              <button 
                className={cn('press-anim-parent', styles.mediaButton, "!min-w-10 !min-h-10 !p-2")}
                onClick={() => onRequestNextSong.onEmit?.()}
              >
                <IconForward className="w-4 h-4 press-anim" />
              </button>
            </div>
            
            {/* 播放列表按钮 */}
            <button
              className={cn('press-anim-parent', styles.mediaButton, "!min-w-10 !min-h-10 !p-2")}
              onClick={() => setPlaylistOpened(v => !v)}
            >
              <MenuIcon className="w-4 h-4 press-anim" />
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
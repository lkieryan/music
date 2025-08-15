import type { FC, PropsWithChildren } from 'react'
import { useState, useRef, useEffect, useCallback } from 'react'
import { SidebarBox } from '../sidebar/sidebar-box'
// splitter removed; resizing is handled by dragging sidebar edges
import { Header } from '../header'
import { useDesktopLayout } from '~/providers/layout-provider'
import { useAtomValue } from 'jotai'
import { playerPlacementAtom, playerVisibleAtom, playerHeightAtom } from '~/atoms/layout'
import { SidebarPlayer } from '../player/sidebar'
import { EssentialsSection } from '../sidebar/essential'
import { MenuShell } from '../sidebar/menu'
import { Outlet } from 'react-router'
import { ContentPlayer } from '../player/content'
import { WindowControlsToolbar } from '../header/toolbar'

// 检测操作系统
const getOS = (): 'windows' | 'mac' | 'linux' => {
  if (typeof window === 'undefined') return 'windows'

  const userAgent = window.navigator.userAgent.toLowerCase()
  if (userAgent.includes('mac')) return 'mac'
  if (userAgent.includes('linux')) return 'linux'
  return 'mac'
}
import { cn } from '~/lib/helper'

const AppContentWrapper: FC<{ rightSide: boolean }> = ({ rightSide }) => {
  const { singleToolbar } = useDesktopLayout()
  const playerPlacement = useAtomValue(playerPlacementAtom)
  const playerVisible = useAtomValue(playerVisibleAtom)
  const playerHeight = useAtomValue(playerHeightAtom)
  return (
    <div className="flex flex-row flex-1 min-h-0 max-h-screen overflow-hidden">
      {rightSide ? (
        <>
          <AppContent rightSide={rightSide} />
          <SidebarBox positionEnd>
            <div className="flex flex-col h-full">
              <div className="flex-1 min-h-0 overflow-y-auto">
                <div className="sticky top-0 z-[2] bg-transparent min-h-[38px] items-stretch">
                  {singleToolbar && (
                    <Header />
                  )}
                </div>
                {/* Essentials comes before dynamic menu (align with Browser layout) */}
                <EssentialsSection />
                {/* Player in middle position (between essentials and menu) */}
                {playerVisible && playerPlacement === 'sidebar-middle' && (
                  <SidebarPlayer height={playerHeight} position="middle" />
                )}
                {/* Desktop menu placed after Essentials */}
                <MenuShell />
              </div>
              {/* Player in bottom position (fixed at sidebar bottom) */}
              {playerVisible && playerPlacement === 'sidebar-bottom' && (
                <SidebarPlayer height={playerHeight} position="bottom" />
              )}
            </div>
          </SidebarBox>
        </>
      ) : (
        <>
          <SidebarBox>
            <div className="flex flex-col h-full">
              <div className="flex-1 min-h-0 overflow-y-auto">
                <div className="sticky top-0 z-[2] bg-transparent min-h-[38px] flex items-center">
                  {singleToolbar ? (
                    <Header />
                  ) : (
                    <div className="flex items-center justify-between w-full h-full px-3">
                      {getOS() === 'mac' && (
                        <WindowControlsToolbar />
                      )}
                    </div>
                  )}
                </div>
                {/* Essentials comes before dynamic menu (align with Browser layout) */}
                <EssentialsSection />
                {/* Player in middle position (between essentials and menu) */}
                {playerVisible && playerPlacement === 'sidebar-middle' && (
                  <SidebarPlayer height={playerHeight} position="middle" />
                )}
                {/* Desktop menu placed after Essentials */}
                <MenuShell />
              </div>
              {/* Player in bottom position (fixed at sidebar bottom) */}
              {playerVisible && playerPlacement === 'sidebar-bottom' && (
                <SidebarPlayer height={playerHeight} position="bottom" />
              )}
            </div>
          </SidebarBox>

          <AppContent rightSide={rightSide} />
        </>
      )}
    </div>
  )
}

const AppContentPanel: FC<PropsWithChildren<{
  tabId: string
  kind?: 'internal' | 'external'
  src?: string
  rightSide?: boolean
}>> = ({ tabId, kind = 'internal', src, children, rightSide }) => {
  const { singleToolbar } = useDesktopLayout()

  return (
    <div className="relative min-h-0 flex-1 overflow-hidden" data-selected-index={0}>
      {/* Backdrop container wrapping the content section */}
      <div className={cn(
        "h-full bg-transparent backdrop-blur-sm backdrop-saturate-[120%] pb-2 flex flex-col",
        singleToolbar && "pt-2",
        rightSide ? "pl-2" : "pr-2"
      )}>
        <section
          className="flex-1 min-h-0 rounded-[10px] bg-white/10 p-2.5 overflow-auto"
          style={{ WebkitOverflowScrolling: 'touch' } as React.CSSProperties}
          data-tab-id={tabId}
        >
          {kind === 'external' && src ? (
            <iframe className="w-full h-full border-0" src={src} title={`tab-${tabId}`} />
          ) : (
            // Use the existing route tree; render nested routes here
            children ?? <Outlet />
          )}
        </section>
      </div>
    </div>
  )
}

// 单工具栏模式下的隐藏窗口控制条
const SingleToolbarWindowControls: FC = () => {
  const [showControls, setShowControls] = useState(false)
  const [isHovering, setIsHovering] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // 清理定时器
  const clearHideTimeout = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
      timeoutRef.current = null
    }
  }, [])

  // 显示窗口控制
  const showWindowControls = useCallback(() => {
    clearHideTimeout()
    setShowControls(true)
  }, [clearHideTimeout])

  // 隐藏窗口控制（延迟）
  const hideWindowControls = useCallback(() => {
    if (!isHovering) {
      clearHideTimeout()
      timeoutRef.current = setTimeout(() => {
        setShowControls(false)
      }, 100) // 快速消失
    }
  }, [isHovering, clearHideTimeout])

  // 监听鼠标位置 - 精确的顶部触发
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      const triggerZone = 8 // 缩小到8px，更接近真正的顶部
      const isInTriggerZone = e.clientY <= triggerZone

      if (isInTriggerZone) {
        showWindowControls()
      } else if (!isHovering) {
        hideWindowControls()
      }
    }

    window.addEventListener('mousemove', handleMouseMove)
    return () => {
      window.removeEventListener('mousemove', handleMouseMove)
      clearHideTimeout()
    }
  }, [isHovering, showWindowControls, hideWindowControls, clearHideTimeout])

  return (
    <div
      className={cn(
        "backdrop-blur-sm backdrop-saturate-[120%] border-b border-black/5 dark:border-white/10",
        "transform-gpu",
        showControls
          ? "h-[32px] opacity-100 translate-y-0 scale-100 transition-all duration-200 ease-out"
          : "h-0 opacity-0 -translate-y-4 scale-95 overflow-hidden pointer-events-none transition-all duration-150 ease-in"
      )}
      data-variant="single-controls"
      onMouseEnter={() => {
        setIsHovering(true)
        showWindowControls()
      }}
      onMouseLeave={() => {
        setIsHovering(false)
        hideWindowControls()
      }}
    >
      <div className="h-[32px] flex items-center">
        <div className="flex items-center justify-between w-full h-full px-3">
          <div className="flex items-center gap-2 text-xs text-text-tertiary opacity-60">
          </div>
          <div className="flex items-center">
            <WindowControlsToolbar />
          </div>
        </div>
      </div>
    </div>
  )
}

export const AppContent: FC<{ rightSide?: boolean }> = ({ rightSide }) => {
  const { singleToolbar } = useDesktopLayout()
  const playerPlacement = useAtomValue(playerPlacementAtom)
  const playerVisible = useAtomValue(playerVisibleAtom)
  const playerHeight = useAtomValue(playerHeightAtom)

  return (
    <div className="flex-1 min-w-0 min-h-0 max-h-full overflow-hidden" data-tabcontainer>
      <div className="flex flex-col h-full min-h-0">
        {/* 多工具栏模式：Windows/Linux显示操作栏，Mac不显示（Mac的控制按钮在侧边栏） */}
        {!singleToolbar && <Header />}
        {/* 单工具栏模式：所有系统都显示隐藏的窗口控制栏 */}
        {singleToolbar && <SingleToolbarWindowControls />}
        <AppContentPanel tabId="t1" rightSide={rightSide} />
        {playerVisible && playerPlacement === 'content-bottom' && (
          <ContentPlayer height={playerHeight} />
        )}
      </div>
    </div>
  )
}


export default AppContentWrapper
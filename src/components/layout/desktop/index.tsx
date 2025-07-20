import type { FC, PropsWithChildren, CSSProperties } from 'react'
import { useAtomValue } from 'jotai'
import { 
  compactModeAtom, 
  playerPlacementAtom, 
  playerVisibleAtom, 
  playerHeightAtom, 
  sidebarPositionAtom, 
  toolbarModeAtom 
} from '~/atoms/layout'
import { LayoutProvider } from '~/providers/layout-provider'
// Header is rendered inside sidebar in single-toolbar mode via TabboxWrapper
import AppContentWrapper from './content'
import { GlobalPlayer } from './player/global'

// TODO: remove this once we have a proper layout system
// import './index.module.css'

// Global layout styles are imported via src/styles/main.css entry. Removing local import avoids order conflicts.

export type ToolbarMode = 'single' | 'multi' | 'compact'
export type SidebarPosition = 'left' | 'right'

export interface LayoutShellProps {
  toolbarMode?: ToolbarMode
  sidebarPosition?: SidebarPosition
  compactMode?: boolean
}

const BaseLayout: FC<PropsWithChildren<{
  rightSide: boolean
  singleToolbar: boolean
  sidebarExpanded?: boolean
  isFullscreen?: boolean
}>> = ({
  rightSide,
  singleToolbar,
  sidebarExpanded = true,
  isFullscreen = false,
  children,
}) => {
  return (
    <div
      className="w-screen h-screen relative overflow-hidden"
      data-right-side={rightSide}
      data-single-toolbar={singleToolbar}
      data-sidebar-expanded={sidebarExpanded}
      data-fullscreen={isFullscreen}
    >
      {/* Background gradient layers - replacing ::before and ::after pseudo-elements */}
      <div 
        className="fixed inset-0 pointer-events-none z-0 transition-opacity duration-300"
        style={{
          background: 'var(--app-background-gradient, var(--main-browser-background, linear-gradient(135deg, #667eea 0%, #764ba2 100%)))',
          opacity: 'var(--app-background-opacity, var(--background-opacity, 1))'
        }}
      />
      <div 
        className="fixed inset-0 pointer-events-none z-0 transition-opacity duration-300"
        style={{
          background: 'var(--app-background-gradient-old, var(--main-browser-background-old, linear-gradient(135deg, #667eea 0%, #764ba2 100%)))',
          opacity: 'calc(1 - var(--app-background-opacity, var(--background-opacity, 1)))'
        }}
      />
      {children}
    </div>
  )
}

export const DesktopLayout: FC<LayoutShellProps> = ({ toolbarMode, sidebarPosition, compactMode }) => {
  // Always read atoms, then decide which source wins to avoid conditional hooks
  const toolbarModeAtomVal = useAtomValue(toolbarModeAtom)
  const sidebarPositionAtomVal = useAtomValue(sidebarPositionAtom)
  const compactModeAtomVal = useAtomValue(compactModeAtom)
  const playerPlacement = useAtomValue(playerPlacementAtom)
  const playerVisible = useAtomValue(playerVisibleAtom)
  const playerHeight = useAtomValue(playerHeightAtom)

  const toolbarModeState = toolbarMode ?? toolbarModeAtomVal
  const sidebarPositionState = sidebarPosition ?? sidebarPositionAtomVal
  const compactModeState = compactMode ?? compactModeAtomVal

  const rightSide = sidebarPositionState === 'right'
  const singleToolbar = toolbarModeState === 'single'
  const effectiveCompact = compactModeState || toolbarModeState === 'compact'

  return (
    <LayoutProvider
      value={{
        rightSide,
        singleToolbar,
        sidebarExpanded: true,
        isFullscreen: false,
        compactMode: effectiveCompact,
      }}
    >
      <BaseLayout rightSide={rightSide} singleToolbar={singleToolbar}>
        {/*
          Structure correction:
          A wrapper contains BOTH toolbar and content as siblings,
          so Header and Content live under the same parent container.
        */}
        <div
          className="flex flex-col h-screen"
          data-compact-mode={effectiveCompact}
          data-player-placement={playerPlacement}
          style={{ ['--player-height' as unknown as keyof CSSProperties]: `${playerHeight}px` } as CSSProperties}
        >
          {/* single toolbar is rendered inside sidebar (see TabboxWrapper) */}
          <div className="relative z-[1] min-w-[1px] flex flex-col flex-1">
            <AppContentWrapper rightSide={rightSide} />
          </div>
          {/* Global bottom player (spans sidebar + content) */}
          {playerVisible && playerPlacement === 'global-bottom' && (
            <GlobalPlayer height={playerHeight} />
          )}
        </div>
      </BaseLayout>
    </LayoutProvider>
  )
}


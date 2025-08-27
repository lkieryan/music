import type { FC } from 'react'
import { useCallback, useEffect, useState } from 'react'
import { cn } from '~/lib/helper'
import { getCurrentWindow } from "@tauri-apps/api/window"

const getOS = (): 'windows' | 'mac' | 'linux' => {
  if (typeof window === 'undefined') return 'windows'
  
  const userAgent = window.navigator.userAgent.toLowerCase()
  if (userAgent.includes('mac')) return 'mac'
  if (userAgent.includes('linux')) return 'linux'
  return 'windows'
}

const ICON_STYLE_CONFIG = {
  forceStyle: 'mac' as 'mac' | 'windows' | null,
  
  allowMacStyleOnWindows: true,
}

const WindowsMinimizeIcon: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} width="12" height="12" viewBox="0 0 12 12" fill="none">
    <rect x="2" y="5.5" width="8" height="1" fill="currentColor" />
  </svg>
)

const WindowsMaximizeIcon: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} width="12" height="12" viewBox="0 0 12 12" fill="none">
    <rect x="2" y="2" width="8" height="8" stroke="currentColor" strokeWidth="1" fill="none" />
  </svg>
)

const WindowsRestoreIcon: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} width="12" height="12" viewBox="0 0 12 12" fill="none">
    <rect x="2" y="3" width="6" height="6" stroke="currentColor" strokeWidth="1" fill="none" />
    <path d="M4 2h6v6" stroke="currentColor" strokeWidth="1" fill="none" />
  </svg>
)

const WindowsCloseIcon: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} width="12" height="12" viewBox="0 0 12 12" fill="none">
    <path d="M2 2L10 10M10 2L2 10" stroke="currentColor" strokeWidth="1" />
  </svg>
)

const MacMinimizeIcon: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} width="8" height="8" viewBox="0 0 8 8" fill="none">
    <rect x="1" y="3.5" width="6" height="1" fill="currentColor" />
  </svg>
)

const MacMaximizeIcon: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} width="8" height="8" viewBox="0 0 8 8" fill="none">
    <path d="M1 3L4 1L7 3V7H1V3Z" stroke="currentColor" strokeWidth="1" fill="none" />
  </svg>
)

const MacCloseIcon: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} width="8" height="8" viewBox="0 0 8 8" fill="none">
    <path d="M2 2L6 6M6 2L2 6" stroke="currentColor" strokeWidth="1.2" />
  </svg>
)

interface WindowControlsProps {
  onMinimize?: () => void
  onMaximize?: () => void
  onClose?: () => void
  className?: string
}

export const WindowControls: FC<WindowControlsProps> = ({
  onMinimize,
  onMaximize,
  onClose,
  className
}) => {
  const [os, setOS] = useState<'windows' | 'mac' | 'linux'>('windows')
  const [isMaximized, setIsMaximized] = useState(false)

  const getIconStyle = (): 'mac' | 'windows' => {
    if (ICON_STYLE_CONFIG.forceStyle) {
      return ICON_STYLE_CONFIG.forceStyle
    }
    
    if (os === 'mac') return 'mac'
    
    return ICON_STYLE_CONFIG.allowMacStyleOnWindows ? 'mac' : 'windows'
  }
  
  const iconStyle = getIconStyle()

  useEffect(() => {
    setOS(getOS())
  
    const webviewWindow = getCurrentWindow()
    let unlisten: (() => void) | undefined

    const listenToEvents = async () => {
      try {
        const maximized = await webviewWindow.isMaximized()
        setIsMaximized(maximized)

        const unlistenFn = await webviewWindow.onResized(async () => {
          const maximized = await webviewWindow.isMaximized()
          setIsMaximized(maximized)
        })
        unlisten = unlistenFn
      } catch (error) {
        console.error("Tauri window event listener setup failed:", error)
      }
    }

    listenToEvents()

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [])

  const handleMinimize = useCallback(() => {
    if (onMinimize) {
      onMinimize()
    } else {
      try {
        getCurrentWindow().minimize()
      } catch (error) {
        console.error('Failed to minimize window:', error)
      }
    }
  }, [onMinimize])

  const handleMaximize = useCallback(() => {
    if (onMaximize) {
      onMaximize()
    } else {
      try {
        getCurrentWindow().toggleMaximize()
      } catch (error) {
        console.error('Failed to toggle maximize window:', error)
      }
    }
  }, [onMaximize])

  const handleClose = useCallback(() => {
    if (onClose) {
      onClose()
    } else {
      try {
        getCurrentWindow().close()
      } catch (error) {
        console.error('Failed to close window:', error)
      }
    }
  }, [onClose])

  if (iconStyle === 'mac') {
    return (
      <div className={cn("flex items-center gap-2 z-[10000] relative", className)}>
        <button
          className="no-drag w-4 h-4 rounded-full bg-[#ff5f57] hover:bg-[#ff4136] flex items-center justify-center group transition-colors"
          onClick={handleClose}
          title="Close"
          aria-label="Close window"
        >
          <MacCloseIcon className="opacity-0 group-hover:opacity-100 transition-opacity text-[#a02622]" />
        </button>
        <button
          className="no-drag w-4 h-4 rounded-full bg-[#ffbd2e] hover:bg-[#ff9500] flex items-center justify-center group transition-colors"
          onClick={handleMinimize}
          title="Minimize"
          aria-label="Minimize window"
        >
          <MacMinimizeIcon className="opacity-0 group-hover:opacity-100 transition-opacity text-[#995700]" />
        </button>
        <button
          className="no-drag w-4 h-4 rounded-full bg-[#28ca42] hover:bg-[#00d642] flex items-center justify-center group transition-colors"
          onClick={handleMaximize}
          title={isMaximized ? "Restore" : "Maximize"}
          aria-label={isMaximized ? "Restore window" : "Maximize window"}
        >
          <MacMaximizeIcon className="opacity-0 group-hover:opacity-100 transition-opacity text-[#0f5d1a]" />
        </button>
      </div>
    )
  }

  return (
    <div className={cn("flex items-center z-[10000] relative", className)}>
      <button
        className="no-drag w-[46px] h-8 flex items-center justify-center hover:bg-[rgba(0,0,0,0.06)] dark:hover:bg-[rgba(255,255,255,0.08)] transition-colors"
        onClick={handleMinimize}
        title="Minimize"
        aria-label="Minimize window"
      >
        <WindowsMinimizeIcon className="text-current opacity-90" />
      </button>
      <button
        className="no-drag w-[46px] h-8 flex items-center justify-center hover:bg-[rgba(0,0,0,0.06)] dark:hover:bg-[rgba(255,255,255,0.08)] transition-colors"
        onClick={handleMaximize}
        title={isMaximized ? "Restore Down" : "Maximize"}
        aria-label={isMaximized ? "Restore window" : "Maximize window"}
      >
        {isMaximized ? (
          <WindowsRestoreIcon className="text-current opacity-90" />
        ) : (
          <WindowsMaximizeIcon className="text-current opacity-90" />
        )}
      </button>
      <button
        className="no-drag w-[46px] h-8 flex items-center justify-center hover:bg-[#e81123] hover:text-white transition-colors"
        onClick={handleClose}
        title="Close"
        aria-label="Close window"
      >
        <WindowsCloseIcon className="text-current opacity-90" />
      </button>
    </div>
  )
}

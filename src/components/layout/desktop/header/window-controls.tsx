import type { FC } from 'react'
import { useCallback, useEffect, useState } from 'react'
import { cn } from '~/lib/helper'
import { getCurrentWindow } from "@tauri-apps/api/window"

// 检测操作系统
const getOS = (): 'windows' | 'mac' | 'linux' => {
  if (typeof window === 'undefined') return 'windows'
  
  const userAgent = window.navigator.userAgent.toLowerCase()
  if (userAgent.includes('mac')) return 'mac'
  if (userAgent.includes('linux')) return 'linux'
  return 'windows'
}

// 🎨 图标风格选择配置 - 可以根据需要修改
const ICON_STYLE_CONFIG = {
  // 强制使用特定风格，如果为null则根据操作系统自动选择
  forceStyle: 'mac' as 'mac' | 'windows' | null, // 默认使用Mac风格图标
  
  // Windows/Linux用户是否允许使用Mac风格（当forceStyle为null时生效）
  allowMacStyleOnWindows: true,
  
  // 可选的配置示例：
  // forceStyle: null,                    // 根据操作系统自动选择
  // forceStyle: 'windows',              // 强制使用Windows风格
  // allowMacStyleOnWindows: false,      // Windows用户强制使用Windows风格
}

// Windows风格的图标
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

// Mac风格的图标 - 严格按照Apple HIG规范，适配12px按钮
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
  
  // 决定使用哪种图标风格
  const getIconStyle = (): 'mac' | 'windows' => {
    if (ICON_STYLE_CONFIG.forceStyle) {
      return ICON_STYLE_CONFIG.forceStyle
    }
    
    // 如果没有强制风格，根据操作系统决定
    if (os === 'mac') return 'mac'
    
    // Windows/Linux用户，如果允许使用Mac风格则使用Mac，否则使用Windows
    return ICON_STYLE_CONFIG.allowMacStyleOnWindows ? 'mac' : 'windows'
  }
  
  const iconStyle = getIconStyle()

  useEffect(() => {
    setOS(getOS())
    
    // 检测窗口是否最大化（在Tauri环境中）
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
    // Mac风格：红黄绿，从左到右，严格按照Apple HIG规范
    // 按钮直径：12px，按钮间距（中心到中心）：20px，所以gap应该是8px (20px - 12px = 8px)
    return (
      <div className={cn("flex items-center gap-2", className)}>
        <button
          className="w-4 h-4 rounded-full bg-[#ff5f57] hover:bg-[#ff4136] flex items-center justify-center group transition-colors"
          onClick={handleClose}
          title="关闭"
          aria-label="关闭窗口"
        >
          <MacCloseIcon className="opacity-0 group-hover:opacity-100 transition-opacity text-[#a02622]" />
        </button>
        <button
          className="w-4 h-4 rounded-full bg-[#ffbd2e] hover:bg-[#ff9500] flex items-center justify-center group transition-colors"
          onClick={handleMinimize}
          title="最小化"
          aria-label="最小化窗口"
        >
          <MacMinimizeIcon className="opacity-0 group-hover:opacity-100 transition-opacity text-[#995700]" />
        </button>
        <button
          className="w-4 h-4 rounded-full bg-[#28ca42] hover:bg-[#00d642] flex items-center justify-center group transition-colors"
          onClick={handleMaximize}
          title={isMaximized ? "还原" : "最大化"}
          aria-label={isMaximized ? "还原窗口" : "最大化窗口"}
        >
          <MacMaximizeIcon className="opacity-0 group-hover:opacity-100 transition-opacity text-[#0f5d1a]" />
        </button>
      </div>
    )
  }

  // Windows/Linux风格：最小化、最大化/还原、关闭，从左到右
  return (
    <div className={cn("flex items-center", className)}>
      <button
        className="w-[46px] h-8 flex items-center justify-center hover:bg-[rgba(0,0,0,0.06)] dark:hover:bg-[rgba(255,255,255,0.08)] transition-colors"
        onClick={handleMinimize}
        title="最小化"
        aria-label="最小化窗口"
      >
        <WindowsMinimizeIcon className="text-current opacity-90" />
      </button>
      <button
        className="w-[46px] h-8 flex items-center justify-center hover:bg-[rgba(0,0,0,0.06)] dark:hover:bg-[rgba(255,255,255,0.08)] transition-colors"
        onClick={handleMaximize}
        title={isMaximized ? "向下还原" : "最大化"}
        aria-label={isMaximized ? "还原窗口" : "最大化窗口"}
      >
        {isMaximized ? (
          <WindowsRestoreIcon className="text-current opacity-90" />
        ) : (
          <WindowsMaximizeIcon className="text-current opacity-90" />
        )}
      </button>
      <button
        className="w-[46px] h-8 flex items-center justify-center hover:bg-[#e81123] hover:text-white transition-colors"
        onClick={handleClose}
        title="关闭"
        aria-label="关闭窗口"
      >
        <WindowsCloseIcon className="text-current opacity-90" />
      </button>
    </div>
  )
}

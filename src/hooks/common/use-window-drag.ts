import { useCallback, useEffect, useRef } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { DEFAULT_NO_DRAG_SELECTOR } from '~/constants/drag'

interface UseWindowDragOptions {
  /**
   * Selector for non-draggable areas, defaults to DEFAULT_NO_DRAG_SELECTOR
   */
  noDragSelector?: string
  /**
   * Whether to enable drag functionality
   */
  enabled?: boolean
  /**
   * Only allow left-click dragging
   */
  leftClickOnly?: boolean
}

/**
 * Window dragging Hook
 * 
 * @param options Configuration options
 * @returns Ref for the draggable area
 */
export const useWindowDrag = <T extends HTMLElement = HTMLDivElement>(options: UseWindowDragOptions = {}) => {
  const {
    noDragSelector = DEFAULT_NO_DRAG_SELECTOR,
    enabled = true,
    leftClickOnly = true
  } = options
  
  const dragRef = useRef<T>(null)

  const handleMouseDown = useCallback(async (e: MouseEvent) => {
    if (!enabled) return
    
    // 只处理左键点击
    if (leftClickOnly && e.button !== 0) return
    
    // 检查是否点击在不可拖拽区域
    const target = e.target as HTMLElement
    if (noDragSelector && target.closest(noDragSelector)) return
    
    try {
      await getCurrentWindow().startDragging()
    } catch (error) {
      console.error('Failed to start window dragging:', error)
    }
  }, [enabled, leftClickOnly, noDragSelector])

  useEffect(() => {
    const element = dragRef.current
    if (!element || !enabled) return

    element.addEventListener('mousedown', handleMouseDown, { capture: true })
    
    return () => {
      element.removeEventListener('mousedown', handleMouseDown, { capture: true })
    }
  }, [handleMouseDown, enabled])

  return dragRef
}
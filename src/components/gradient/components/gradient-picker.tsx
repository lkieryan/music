import React, { useRef, useCallback, useEffect } from 'react'
import type { ColorDot } from '~/types/gradient'
import { PICKER_SIZE, PICKER_PADDING, DOT_SIZE, PRIMARY_DOT_SIZE } from '~/constants/gradient'

interface GradientPickerProps {
  dots: ColorDot[]
  onDotClick: (position: { x: number; y: number }) => void
  onDotDrag: (dotId: number, position: { x: number; y: number }) => void
  onDotRemove: (dotId: number) => void
  dragging: boolean
  setDragging: (dragging: boolean) => void
  draggedDot: HTMLElement | null
  setDraggedDot: (element: HTMLElement | null) => void
  currentLightness: number
  className?: string
}

export default function GradientPicker({
  dots,
  onDotClick,
  onDotDrag,
  onDotRemove,
  dragging,
  setDragging,
  draggedDot,
  setDraggedDot,
  // currentLightness, // 未使用
  className = '',
}: GradientPickerProps) {
  const pickerRef = useRef<HTMLDivElement>(null)
  const recentlyDragged = useRef(false)
  
  // 性能优化：节流拖动更新
  const dragUpdateRef = useRef<number | null>(null)
  const lastDragPosition = useRef<{ x: number; y: number } | null>(null)

  const handleMouseDown = useCallback((event: React.MouseEvent<HTMLDivElement>) => {
    const target = event.target as HTMLElement
    
    if (target.classList.contains('theme-picker-dot')) {
      event.preventDefault()
      setDragging(true)
      setDraggedDot(target)
      target.setAttribute('data-dragging', 'true')
    }
  }, [setDragging, setDraggedDot])

  const handleMouseMove = useCallback((event: MouseEvent) => {
    if (!dragging || !draggedDot || !pickerRef.current) return

    event.preventDefault()
    const rect = pickerRef.current.getBoundingClientRect()
    const padding = PICKER_PADDING
    
    // Constrain to circular boundary
    const centerX = rect.left + rect.width / 2
    const centerY = rect.top + rect.height / 2
    const radius = (rect.width - padding) / 2
    let pixelX = event.clientX
    let pixelY = event.clientY
    
    const distance = Math.sqrt((pixelX - centerX) ** 2 + (pixelY - centerY) ** 2)
    if (distance > radius) {
      const angle = Math.atan2(pixelY - centerY, pixelX - centerX)
      pixelX = centerX + Math.cos(angle) * radius
      pixelY = centerY + Math.sin(angle) * radius
    }

    const relativeX = pixelX - rect.left
    const relativeY = pixelY - rect.top

    // 🚀 性能优化：立即更新视觉位置（60fps）
    draggedDot.style.left = `${relativeX}px`
    draggedDot.style.top = `${relativeY}px`

    // 🚀 性能优化：节流状态更新（避免过度重新渲染）
    lastDragPosition.current = { x: relativeX, y: relativeY }
    
    if (dragUpdateRef.current) {
      cancelAnimationFrame(dragUpdateRef.current)
    }
    
    dragUpdateRef.current = requestAnimationFrame(() => {
      if (lastDragPosition.current && dragging && draggedDot) {
        const dotId = parseInt(draggedDot.getAttribute('data-dot-id') || '0')
        onDotDrag(dotId, lastDragPosition.current)
      }
      dragUpdateRef.current = null
    })
  }, [dragging, draggedDot, onDotDrag])

  const handleMouseUp = useCallback((event: MouseEvent) => {
    if (event.button === 2 && draggedDot) {
      // Right click to remove dot
      const dotId = parseInt(draggedDot.getAttribute('data-dot-id') || '0')
      onDotRemove(dotId)
      return
    }

    if (dragging && draggedDot) {
      event.preventDefault()
      event.stopPropagation()
      
      // 🚀 性能优化：清理待处理的RAF
      if (dragUpdateRef.current) {
        cancelAnimationFrame(dragUpdateRef.current)
        dragUpdateRef.current = null
      }
      
      // 确保最后一次位置更新
      if (lastDragPosition.current) {
        const dotId = parseInt(draggedDot.getAttribute('data-dot-id') || '0')
        onDotDrag(dotId, lastDragPosition.current)
        lastDragPosition.current = null
      }
      
      setDragging(false)
      draggedDot.removeAttribute('data-dragging')
      setDraggedDot(null)

      recentlyDragged.current = true
      setTimeout(() => {
        recentlyDragged.current = false
      }, 100)
    }
  }, [dragging, draggedDot, setDragging, setDraggedDot, onDotRemove, onDotDrag])

  const handleClick = useCallback((event: React.MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0 || dragging || recentlyDragged.current) return
    if (!pickerRef.current) return

    const target = event.target as HTMLElement
    if (target.classList.contains('theme-picker-dot')) return

    const rect = pickerRef.current.getBoundingClientRect()
    const padding = PICKER_PADDING

    const centerX = rect.left + rect.width / 2
    const centerY = rect.top + rect.height / 2
    const radius = (rect.width - padding) / 2
    let pixelX = event.clientX
    let pixelY = event.clientY

    const distance = Math.sqrt((pixelX - centerX) ** 2 + (pixelY - centerY) ** 2)
    if (distance > radius) {
      const angle = Math.atan2(pixelY - centerY, pixelX - centerX)
      pixelX = centerX + Math.cos(angle) * radius
      pixelY = centerY + Math.sin(angle) * radius
    }

    const relativeX = pixelX - rect.left
    const relativeY = pixelY - rect.top

    onDotClick({ x: relativeX, y: relativeY })
  }, [dragging, onDotClick])

  // Global mouse events
  useEffect(() => {
    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [handleMouseMove, handleMouseUp])
  
  // 🚀 性能优化：组件卸载时清理RAF
  useEffect(() => {
    return () => {
      if (dragUpdateRef.current) {
        cancelAnimationFrame(dragUpdateRef.current)
      }
    }
  }, [])

  return (
    <div
      ref={pickerRef}
      className={`relative overflow-hidden rounded-lg bg-black/5 dark:bg-white/[0.03] ${className}`}
      onMouseDown={handleMouseDown}
      onClick={handleClick}
      onContextMenu={(e) => e.preventDefault()}
      style={{
        minHeight: `${PICKER_SIZE - 20}px`,
        backgroundImage: `radial-gradient(rgba(0, 0, 0, 0.2) 1px, transparent 0)`,
        backgroundPosition: '-23px -23px',
        backgroundSize: '6px 6px',
      }}
    >
      {dots.map((dot) => {
        if (!dot.position) return null
        
        const [r, g, b] = Array.isArray(dot.c) ? dot.c : [0, 0, 0]
        const isPrimary = dot.isPrimary || dot.ID === 0
        
        return (
          <div
            key={dot.ID}
            className={`theme-picker-dot ${isPrimary ? 'primary' : ''}`}
            aria-hidden={isPrimary ? 'false' : 'true'}
            data-dot-id={dot.ID}
            data-position={JSON.stringify({ x: Math.round(dot.position.x), y: Math.round(dot.position.y) })}
            data-type={dot.type}
            data-dragging={dragging && draggedDot?.getAttribute('data-dot-id') === dot.ID.toString() ? 'true' : 'false'}
            style={{
              position: 'absolute',
              zIndex: isPrimary ? 999 : 2,
              width: isPrimary ? `${PRIMARY_DOT_SIZE}px` : `${DOT_SIZE}px`,
              height: isPrimary ? `${PRIMARY_DOT_SIZE}px` : `${DOT_SIZE}px`,
              borderRadius: '50%',
              background: `rgb(${r}, ${g}, ${b})`,
              cursor: 'pointer',
              border: isPrimary ? '4px solid #ffffff' : '3px solid #ffffff',
              left: `${dot.position.x}px`,
              top: `${dot.position.y}px`,
              transform: 'translate(-50%, -50%)',
              pointerEvents: isPrimary ? 'all' : 'none',
              transformOrigin: 'center center',
              transition: dragging ? 'none' : 'transform 0.2s',
              boxShadow: 'rgba(0, 0, 0, 0.1) 0px 0px 0px 2px',
              '--theme-picker-dot-color': `rgb(${r}, ${g}, ${b})`,
            } as React.CSSProperties}
          />
        )
      })}
      
      {dots.length === 0 && (
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 whitespace-nowrap pointer-events-none text-sm font-semibold m-0">
        </div>
      )}
    </div>
  )
}
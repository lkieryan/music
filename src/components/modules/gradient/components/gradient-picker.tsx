import React, { useRef, useCallback, useEffect } from 'react'
import type { CSSProperties } from 'react'
import type { ColorDot } from '~/types/gradient'
import { PICKER_SIZE, PICKER_PADDING, DOT_SIZE, PRIMARY_DOT_SIZE } from '../constants'

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
  currentLightness,
  className = '',
}: GradientPickerProps) {
  const pickerRef = useRef<HTMLDivElement>(null)
  const recentlyDragged = useRef(false)

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

    // Update dot position
    draggedDot.style.left = `${relativeX}px`
    draggedDot.style.top = `${relativeY}px`

    // Find dot ID and update position
    const dotId = parseInt(draggedDot.getAttribute('data-dot-id') || '0')
    onDotDrag(dotId, { x: relativeX, y: relativeY })
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
      setDragging(false)
      draggedDot.removeAttribute('data-dragging')
      setDraggedDot(null)

      recentlyDragged.current = true
      setTimeout(() => {
        recentlyDragged.current = false
      }, 100)
    }
  }, [dragging, draggedDot, setDragging, setDraggedDot, onDotRemove])

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

  return (
    <div
      ref={pickerRef}
      className={`theme-picker-gradient ${className}`}
      onMouseDown={handleMouseDown}
      onClick={handleClick}
      onContextMenu={(e) => e.preventDefault()}
      style={{
        position: 'relative',
        overflow: 'hidden',
        borderRadius: '8px',
        minHeight: `${PICKER_SIZE - 20}px`,
        background: 'light-dark(rgba(0, 0, 0, 0.05), rgba(255, 255, 255, 0.03))',
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
            style={(() => {
              const dotStyle: CSSProperties & { ['--theme-picker-dot-color']?: string } = {
                position: 'absolute',
                zIndex: isPrimary ? 2 : 2,
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
                ['--theme-picker-dot-color']: `rgb(${r}, ${g}, ${b})`,
              }
              return dotStyle
            })()}
          />
        )
      })}
      
      {dots.length === 0 && (
        <div
          style={{
            position: 'absolute',
            fontWeight: 600,
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            whiteSpace: 'nowrap',
            pointerEvents: 'none',
            fontSize: 'small',
            margin: 0,
          }}
        >
        </div>
      )}
    </div>
  )
}
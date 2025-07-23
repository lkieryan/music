import React, { useRef, useCallback, useEffect } from 'react'

interface TextureSelectorProps {
  value: number
  onChange: (value: number) => void
  className?: string
}

export default function TextureSelector({ value, onChange, className = '' }: TextureSelectorProps) {
  const wrapperRef = useRef<HTMLDivElement>(null)
  const isDragging = useRef(false)

  const handleMouseDown = useCallback((event: React.MouseEvent) => {
    event.preventDefault()
    isDragging.current = true
  }, [])

  const handleMouseMove = useCallback((event: MouseEvent) => {
    if (!isDragging.current || !wrapperRef.current) return

    event.preventDefault()
    const wrapperRect = wrapperRef.current.getBoundingClientRect()
    
    // Determine rotation based on mouse position and center of wrapper
    const rotation = Math.atan2(
      event.clientY - wrapperRect.top - wrapperRect.height / 2,
      event.clientX - wrapperRect.left - wrapperRect.width / 2
    )
    
    const previousTexture = value
    let currentTexture = (rotation * 180) / Math.PI + 90
    
    // Make it positive if negative
    if (currentTexture < 0) {
      currentTexture += 360
    }
    
    // Convert from degrees to 0-1 range
    currentTexture /= 360
    
    // Clip to closest button out of 16 possible buttons
    currentTexture = Math.round(currentTexture * 16) / 16
    if (currentTexture === 1) {
      currentTexture = 0
    }
    
    if (previousTexture !== currentTexture) {
      onChange(currentTexture)
    }
  }, [value, onChange])

  const handleMouseUp = useCallback((event: MouseEvent) => {
    event.preventDefault()
    isDragging.current = false
  }, [])

  useEffect(() => {
    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [handleMouseMove, handleMouseUp])

  // Create 16 texture dots in circular pattern
  const textureDots = Array.from({ length: 16 }, (_, i) => {
    const angle = (i / 16) * Math.PI * 2
    const x = Math.cos(angle) * 50 + 50
    const y = Math.sin(angle) * 50 + 50
    
    // Check if this dot should be active based on texture value
    let adjustedIndex = i + 4 // Start at point 4 as per original
    if (adjustedIndex >= 16) adjustedIndex -= 16
    const isActive = (adjustedIndex / 16) <= value
    
    return (
      <div
        key={i}
        className={`theme-picker-texture-dot ${isActive ? 'active' : ''}`}
        style={{
          height: '4px',
          width: '4px',
          borderRadius: '50%',
          background: 'rgba(0, 0, 0, 0.3)',
          position: 'absolute',
          transition: 'opacity 0.2s',
          transform: 'translate(-50%, -50%)',
          pointerEvents: 'none',
          opacity: isActive ? 1 : 0.4,
          left: `${x}%`,
          top: `${y}%`,
        }}
      />
    )
  })

  // Calculate handler position
  const handlerRotation = value * 360 - 90
  const handlerTop = Math.sin((handlerRotation * Math.PI) / 180) * 50
  const handlerLeft = Math.cos((handlerRotation * Math.PI) / 180) * 50

  return (
    <div
      ref={wrapperRef}
      className={`gradient-texture-wrapper ${className}`}
      style={{
        width: '4rem',
        height: '4rem',
        position: 'relative',
        right: '15px',
      }}
    >
      {/* Background grain effect */}
      <div
        style={{
          position: 'absolute',
          width: '60%',
          height: '60%',
          opacity: value,
          mixBlendMode: 'hard-light',
          pointerEvents: 'none',
          top: '50%',
          borderRadius: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          zIndex: 1,
          background: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E")`,
        }}
      />
      
      {/* Border circle */}
      <div
        style={{
          position: 'absolute',
          width: '60%',
          height: '60%',
          border: '1px solid rgba(0, 0, 0, 0.2)',
          borderRadius: '50%',
          background: 'linear-gradient(-45deg, transparent -10%, rgba(0, 0, 0, 0.1) 110%)',
          zIndex: 2,
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          pointerEvents: 'none',
        }}
      />
      
      {/* Texture dots */}
      {textureDots}
      
      {/* Handler */}
      <div
        onMouseDown={handleMouseDown}
        style={{
          width: '6px',
          height: '12px',
          background: '#757575',
          position: 'absolute',
          transition: 'height 0.1s',
          zIndex: 2,
          borderRadius: '2px',
          cursor: 'pointer',
          transform: `rotate(${handlerRotation + 90}deg)`,
          top: `${50 + handlerTop}%`,
          left: `${50 + handlerLeft}%`,
          transformOrigin: 'center center',
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.height = '14px'
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.height = '12px'
        }}
      />
    </div>
  )
}
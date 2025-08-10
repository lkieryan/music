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
        className={`h-1 w-1 rounded-full bg-black/30 dark:bg-white/30 absolute transition-opacity duration-200 -translate-x-1/2 -translate-y-1/2 pointer-events-none ${
          isActive ? 'opacity-100' : 'opacity-40'
        }`}
        style={{
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
      className={`w-16 h-16 relative -right-[15px] ${className}`}
    >
      {/* Background grain effect */}
      <div
        className="absolute w-3/5 h-3/5 pointer-events-none top-1/2 rounded-full left-1/2 -translate-x-1/2 -translate-y-1/2 z-[1]"
        style={{
          opacity: value,
          mixBlendMode: 'hard-light',
          background: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E")`,
        }}
      />
      
      {/* Border circle */}
      <div
        className="absolute w-3/5 h-3/5 border border-black/20 dark:border-white/20 rounded-full z-[2] top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none"
        style={{
          background: 'linear-gradient(-45deg, transparent -10%, rgba(0, 0, 0, 0.1) 110%)',
        }}
      />
      
      {/* Texture dots */}
      {textureDots}
      
      {/* Handler */}
      <div
        onMouseDown={handleMouseDown}
        className="w-1.5 h-3 bg-gray-600 dark:bg-gray-400 absolute transition-[height] duration-100 z-[2] rounded-sm cursor-pointer origin-center hover:h-3.5"
        style={{
          transform: `rotate(${handlerRotation + 90}deg)`,
          top: `${50 + handlerTop}%`,
          left: `${50 + handlerLeft}%`,
          transformOrigin: 'center center',
        }}
      />
    </div>
  )
}
import React from 'react'
import { MAX_OPACITY, MIN_OPACITY } from '../constants'
import { interpolateWavePath } from '~/lib/svg'

interface OpacitySliderProps {
  value: number
  onChange: (value: number) => void
  disabled?: boolean
  className?: string
  hideThumb?: boolean
}

export default function OpacitySlider({ value, onChange, disabled = false, className = '', hideThumb = false }: OpacitySliderProps) {
  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = parseFloat(event.target.value)
    onChange(newValue)
  }

  // Calculate wave interpolation progress
  const opacity = hideThumb ? 1 : Math.min(1, Math.max(0, (value - MIN_OPACITY) / (MAX_OPACITY - MIN_OPACITY)))

  const interpolatedPath = interpolateWavePath(opacity)
  const thumbHeight = 40 + opacity * 15
  const thumbWidth = 10 + opacity * 15

  return (
    <div className={`relative ${className}`}>
      <div
        id="gradient-slider-wave"
        className="absolute -left-1.5 w-full h-full pointer-events-none z-[1] flex items-center justify-start"
      >
        <div className="absolute w-[calc(100%-8px)] h-4 bg-black/10 dark:bg-white/10 rounded-full pointer-events-none -z-[1] top-1/2 left-2 -translate-y-1/2" />
        <svg
          viewBox="0 -7.605 455 70"
          xmlns="http://www.w3.org/2000/svg"
          preserveAspectRatio="xMinYMid meet"
          className="overflow-visible ml-1 scale-[1.2]"
          style={{
            minWidth: 'calc(100% * 1.1)',
          }}
        >
          <path
            d={interpolatedPath}
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
            style={{
              strokeWidth: '8px',
              stroke: interpolatedPath === 'M 51.373 27.395 L 367.037 27.395' 
                ? 'rgba(77, 77, 77, 0.5)' 
                : 'url(#gradient-generator-slider-wave-gradient)',
            }}
          />
          <defs>
            <linearGradient id="gradient-generator-slider-wave-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="rgb(90, 90, 90)" />
              <stop offset={`${opacity * 100}%`} stopColor="rgb(90, 90, 90)" />
              <stop offset={`${opacity * 100}%`} stopColor="rgba(77, 77, 77, 0.5)" />
            </linearGradient>
          </defs>
        </svg>
      </div>
      
      {
        !hideThumb && (
          <input
            type="range"
            min={MIN_OPACITY}
            max={MAX_OPACITY}
            step="0.001"
            value={value}
            onChange={handleChange}
            disabled={disabled}
            className="gradient-opacity-slider !m-0 bg-transparent z-[2] px-1.5 w-full"
            style={{
              '--thumb-height': `${thumbHeight}px`,
              '--thumb-width': `${thumbWidth}px`,
            } as React.CSSProperties}
          />
        )
      }
      
    </div>
  )
}
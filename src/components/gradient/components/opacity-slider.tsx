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
    <div className={`gradient-opacity-wrapper ${className}`} style={{ position: 'relative' }}>
      <div
        id="gradient-slider-wave"
        style={{
          position: 'absolute',
          left: '-5px',
          width: '100%',
          height: '100%',
          pointerEvents: 'none',
          zIndex: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-start',
        }}
      >
        <div
          style={{
            position: 'absolute',
            width: 'calc(100% - 8px)',
            height: '16px',
            background: 'rgba(0, 0, 0, 0.1)',
            borderRadius: '999px',
            pointerEvents: 'none',
            zIndex: -1,
            top: '50%',
            left: '8px',
            transform: 'translateY(-50%)',
          }}
        />
        <svg
          viewBox="0 -7.605 455 70"
          xmlns="http://www.w3.org/2000/svg"
          preserveAspectRatio="xMinYMid meet"
          style={{
            overflow: 'visible',
            minWidth: 'calc(100% * 1.1)',
            scale: '1.2',
            marginLeft: '4px',
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
            style={{
              margin: '0 !important',
              background: 'transparent',
              zIndex: 2,
              padding: '0 5px',
              width: '100%',
              '--thumb-height': `${thumbHeight}px`,
              '--thumb-width': `${thumbWidth}px`,
            }}
            className="gradient-opacity-slider"
          />
        )
      }
      
    </div>
  )
}
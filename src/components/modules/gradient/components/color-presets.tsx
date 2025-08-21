import React from 'react'
import { PRESET_COLORS } from '../constants'

interface ColorPresetsProps {
  currentPage: number
  onPageChange: (page: number) => void
  onPresetSelect: (lightness: number, algo: string, numDots: number, position: string, colors?: string[]) => void
  className?: string
}

// Preset types
// Single color preset with inline CSS style
interface SinglePreset {
  lightness: number
  algo: string
  numDots: 1
  position: string
  style: string
}

// Analogous preset with three explicit colors
interface AnalogousPreset {
  lightness: number
  algo: string
  numDots: 3
  position: string
  colors: string[]
}

type Preset = SinglePreset | AnalogousPreset

// Type guard to check if a preset is AnalogousPreset
function isAnalogousPreset(preset: Preset): preset is AnalogousPreset {
  return preset.numDots === 3 && 'colors' in preset
}

export default function ColorPresets({ 
  currentPage, 
  onPageChange, 
  onPresetSelect, 
  className = '' 
}: ColorPresetsProps) {
  const totalPages = PRESET_COLORS.length

  const handlePresetClick = (preset: Preset) => {
    onPresetSelect(
      preset.lightness,
      preset.algo,
      preset.numDots,
      preset.position,
      isAnalogousPreset(preset) ? preset.colors : undefined
    )
  }

  const goToPreviousPage = () => {
    const newPage = (currentPage - 1 + totalPages) % totalPages
    onPageChange(newPage)
  }

  const goToNextPage = () => {
    const newPage = (currentPage + 1) % totalPages
    onPageChange(newPage)
  }

  // Cast the data source into our Preset type for this module's usage
  const currentPresets = (PRESET_COLORS[currentPage] || []) as Preset[]

  return (
    <div className={`gradient-color-pages-wrapper ${className}`} style={{ alignItems: 'center', display: 'flex' }}>
      <button
        onClick={goToPreviousPage}
        disabled={currentPage === 0}
        style={{
          maxWidth: '28px',
          maxHeight: '28px',
          margin: '0',
          border: 'none',
          background: 'rgba(0, 0, 0, 0.1)',
          borderRadius: '4px',
          cursor: currentPage === 0 ? 'not-allowed' : 'pointer',
          opacity: currentPage === 0 ? 0.5 : 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '12px',
        }}
        aria-label="Previous page"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.41,16.58L10.83,12L15.41,7.41L14,6L8,12L14,18L15.41,16.58Z" />
        </svg>
      </button>

      <div
        style={{
          display: 'flex',
          justifyContent: 'space-around',
          margin: '0 10px',
          alignItems: 'center',
          width: '100%',
          overflow: 'auto',
          scrollbarWidth: 'none',
          scrollBehavior: 'smooth',
          maskImage: 'linear-gradient(to right, transparent 0%, black 2.5%, black 97.5%, transparent 100%)',
        }}
      >
        <div style={{ 
          justifyContent: 'space-around', 
          minWidth: '100%',
          display: 'flex',
          gap: '8px',
        }}>
          {currentPresets.map((preset, index) => {
            let backgroundStyle: React.CSSProperties = {}

            if (isAnalogousPreset(preset)) {
              // Multi-color analogous preset
              backgroundStyle = {
                background: [
                  `radial-gradient(circle at 0% 0%, ${preset.colors[0]}, transparent 100%)`,
                  `radial-gradient(circle at 100% 0%, ${preset.colors[1]}, transparent 100%)`,
                  `linear-gradient(to top, ${preset.colors[2]} 0%, transparent 60%)`
                ].join(', ')
              }
            } else if ('style' in preset) {
              // Single color preset
              const colorMatch = preset.style.match(/background:\s*([^;]+);?/)
              if (colorMatch) {
                backgroundStyle.background = colorMatch[1]
              }
            }

            return (
              <div
                key={index}
                onClick={() => handlePresetClick(preset)}
                data-lightness={preset.lightness}
                data-algo={preset.algo}
                data-num-dots={preset.numDots}
                data-position={preset.position}
                style={{
                  width: '22px',
                  height: '22px',
                  borderRadius: '50%',
                  cursor: 'pointer',
                  position: 'relative',
                  transition: 'transform 0.1s',
                  ...backgroundStyle,
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'scale(1.05)'
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'scale(1)'
                }}
                onMouseDown={(e) => {
                  e.currentTarget.style.transform = 'scale(0.95)'
                }}
                onMouseUp={(e) => {
                  e.currentTarget.style.transform = 'scale(1.05)'
                }}
              />
            )
          })}
        </div>
      </div>

      <button
        onClick={goToNextPage}
        disabled={currentPage === totalPages - 1}
        style={{
          maxWidth: '28px',
          maxHeight: '28px',
          margin: '0',
          border: 'none',
          background: 'rgba(0, 0, 0, 0.1)',
          borderRadius: '4px',
          cursor: currentPage === totalPages - 1 ? 'not-allowed' : 'pointer',
          opacity: currentPage === totalPages - 1 ? 0.5 : 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '12px',
        }}
        aria-label="Next page"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z" />
        </svg>
      </button>
    </div>
  )
}
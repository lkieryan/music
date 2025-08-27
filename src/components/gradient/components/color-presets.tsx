import React from 'react'
import { PRESET_COLORS } from '~/constants/gradient'

interface ColorPresetsProps {
  currentPage: number
  onPageChange: (page: number) => void
  onPresetSelect: (lightness: number, algo: string, numDots: number, position: string) => void
  className?: string
}

export default function ColorPresets({ 
  currentPage, 
  onPageChange, 
  onPresetSelect, 
  className = '' 
}: ColorPresetsProps) {
  const totalPages = PRESET_COLORS.length

  const handlePresetClick = (preset: typeof PRESET_COLORS[0][0]) => {
    onPresetSelect(
      preset.lightness,
      preset.algo,
      preset.numDots,
      preset.position
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

  const currentPresets = PRESET_COLORS[currentPage] || []

  return (
    <div className={`items-center flex ${className}`}>
      <button
        onClick={goToPreviousPage}
        disabled={currentPage === 0}
        className={`
          max-w-7 max-h-7 m-0 border-none bg-black/10 rounded 
          flex items-center justify-center text-xs
          ${currentPage === 0 
            ? 'cursor-not-allowed opacity-50' 
            : 'cursor-pointer opacity-100'
          }
        `}
        aria-label="Previous page"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.41,16.58L10.83,12L15.41,7.41L14,6L8,12L14,18L15.41,16.58Z" />
        </svg>
      </button>

      <div 
        className="flex justify-around mx-2.5 items-center w-full overflow-auto scroll-smooth"
        style={{
          scrollbarWidth: 'none',
          maskImage: 'linear-gradient(to right, transparent 0%, black 2.5%, black 97.5%, transparent 100%)',
        }}
      >
        <div className="justify-around min-w-full flex gap-2">
          {currentPresets.map((preset, index) => {
            const isAnalogous = preset.numDots === 3
            
            let backgroundStyle: React.CSSProperties = {}
            
            if (isAnalogous && 'colors' in preset && preset.colors) {
              // Multi-color analogous preset
              backgroundStyle = {
                background: [
                  `radial-gradient(circle at 0% 0%, ${preset.colors[0]}, transparent 100%)`,
                  `radial-gradient(circle at 100% 0%, ${preset.colors[1]}, transparent 100%)`,
                  `linear-gradient(to top, ${preset.colors[2]} 0%, transparent 60%)`
                ].join(', ')
              }
            } else if ('style' in preset && preset.style) {
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
                className="w-[22px] h-[22px] rounded-full cursor-pointer relative transition-transform duration-100 hover:scale-105 active:scale-95"
                style={{
                  ...backgroundStyle,
                }}
              />
            )
          })}
        </div>
      </div>

      <button
        onClick={goToNextPage}
        disabled={currentPage === totalPages - 1}
        className={`
          max-w-7 max-h-7 m-0 border-none bg-black/10 rounded 
          flex items-center justify-center text-xs
          ${currentPage === totalPages - 1 
            ? 'cursor-not-allowed opacity-50' 
            : 'cursor-pointer opacity-100'
          }
        `}
        aria-label="Next page"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z" />
        </svg>
      </button>
    </div>
  )
}
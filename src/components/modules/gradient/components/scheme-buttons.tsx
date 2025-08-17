import React from 'react'
import SparklesIcon from '~/assets/icons/sparkles.svg?react'
import FaceSunIcon from '~/assets/icons/face-sun.svg?react'
import MoonStarsIcon from '~/assets/icons/moon-stars.svg?react'

interface SchemeButtonsProps {
  currentScheme: 'auto' | 'light' | 'dark'
  onSchemeChange: (scheme: 'auto' | 'light' | 'dark') => void
  className?: string
}

export default function SchemeButtons({
  currentScheme,
  onSchemeChange,
  className = ''
}: SchemeButtonsProps) {
  const schemes = [
    { id: 'auto', svg: <SparklesIcon width={16} height={16} />, title: 'Auto' },
    { id: 'light', svg: <FaceSunIcon width={16} height={16} />, title: 'Light' },
    { id: 'dark', svg: <MoonStarsIcon width={16} height={16} />, title: 'Dark' },
  ] as const

  return (
    <div
      className={`gradient-scheme ${className}`}
      style={{
        display: 'flex',
        position: 'absolute',
        top: '20px',
        left: '50%',
        zIndex: 3,
        transform: 'translateX(-50%)',
        gap: '5px',
        maxHeight: '32px',
      }}
      onMouseDown={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
      onPointerDown={(e) => e.stopPropagation()}
    >
      {schemes.map((scheme) => (
        <button
          key={scheme.id}
          onClick={() => onSchemeChange(scheme.id)}
          style={{
            border: 'none',
            padding: '0',
            minWidth: 'fit-content',
            transition: 'background 0.2s',
            appearance: 'none',
            maxHeight: '26px',
            maxWidth: '26px',
            minHeight: '26px',
            color: currentScheme === scheme.id 
              ? 'rgba(0, 0, 0, 1)' 
              : 'rgba(0, 0, 0, 0.7)',
            background: currentScheme === scheme.id 
              ? 'rgba(0, 0, 0, 0.15)' 
              : 'transparent',
            borderRadius: '4px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '14px',
            fontWeight: 'bold',
          }}
          onMouseEnter={(e) => {
            if (currentScheme !== scheme.id) {
              e.currentTarget.style.background = 'rgba(0, 0, 0, 0.1)'
            }
          }}
          onMouseLeave={(e) => {
            if (currentScheme !== scheme.id) {
              e.currentTarget.style.background = 'transparent'
            }
          }}
          title={scheme.title}
        >
          {scheme.svg}
        </button>
      ))}
    </div>
  )
}
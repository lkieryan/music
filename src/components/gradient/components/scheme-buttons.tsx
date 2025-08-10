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
      className={`flex absolute top-5 left-1/2 z-[3] -translate-x-1/2 gap-1.5 max-h-8 ${className}`}
      onMouseDown={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
      onPointerDown={(e) => e.stopPropagation()}
    >
      {schemes.map((scheme) => (
        <button
          key={scheme.id}
          onClick={() => onSchemeChange(scheme.id)}
          className={`
            border-none p-0 transition-colors duration-200 appearance-none 
            h-[26px] w-[26px] rounded cursor-pointer 
            flex items-center justify-center text-sm font-bold
            ${currentScheme === scheme.id 
              ? 'text-gray-900 dark:text-gray-100 bg-black/15 dark:bg-white/15' 
              : 'text-gray-700 dark:text-gray-300 bg-transparent hover:bg-black/10 dark:hover:bg-white/10'
            }
          `}
          title={scheme.title}
        >
          {scheme.svg}
        </button>
      ))}
    </div>
  )
}
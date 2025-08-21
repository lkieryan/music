import type { FC } from 'react'
import SearchIcon from '~/assets/icons/search-glass.svg?react'
import { useDesktopLayout } from '~/providers/layout-provider'
import { cn } from '~/lib/helper'

export const Urlbar: FC = () => {
  const { singleToolbar } = useDesktopLayout()
  
  return (
    <div 
      className={cn(
        "flex items-center w-full",
        singleToolbar ? "h-10" : "h-8"
      )}
      role="search" 
      aria-label="Address and search bar"
    >
      <div className="grid grid-cols-[24px_1fr] items-center gap-1.5 h-full w-full bg-[rgba(0,0,0,0.06)] dark:bg-[rgba(255,255,255,0.08)] rounded-[8px] px-2">
        <span className="inline-flex items-center justify-center text-current opacity-90" aria-hidden>
          <SearchIcon 
            className="[&_*[stroke='context-fill']]:stroke-current [&_*[fill='context-fill']]:fill-current [&_*[stroke-opacity='context-fill-opacity']]:stroke-opacity-100" 
            width={16} 
            height={16} 
          />
        </span>
        <input 
          className="h-full w-full border-none outline-none rounded-[8px] bg-transparent text-text px-0.5 placeholder:text-text-tertiary" 
          placeholder="Search or enter address" 
        />
      </div>
    </div>
  )
}


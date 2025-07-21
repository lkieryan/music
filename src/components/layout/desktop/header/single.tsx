import type { FC } from 'react'
import { useAtomValue } from 'jotai'
import { sidebarPositionAtom } from '~/atoms/layout'
import { Urlbar } from '../sidebar/urlbar'
import { LeftToolbar, RightToolbar } from './toolbar'
import { cn } from '~/lib/helper'

// Header for single-toolbar mode: rendered inside SidebarBox header
export const HeaderSingle: FC = () => {
  const rightSide = useAtomValue(sidebarPositionAtom) === 'right'
  return (
    <div 
      className="bg-transparent h-auto" 
      data-variant="single"
      style={{ '--urlbar-height': '40px' } as React.CSSProperties}
    >
      <div className="grid auto-rows-min gap-1.5 px-1.5 py-1">
        <div 
          className={cn(
            "grid items-center h-[38px] gap-3",
            !rightSide && "grid-cols-[auto_1fr_auto]",
            rightSide && "grid-cols-[auto_auto_1fr]"
          )}
          data-right-side={rightSide || undefined}
        >
          {rightSide ? (
            <>
              <div className="flex items-center" aria-hidden>
                <RightToolbar />
              </div>
              <div className="flex items-center" aria-hidden>
                <LeftToolbar />
              </div>
              <div className="min-w-px" />
            </>
          ) : (
            <>
              <div className="flex items-center" aria-hidden>
                <LeftToolbar />
              </div>
              <div className="min-w-px" />
              <div className="flex items-center" aria-hidden>
                <RightToolbar />
              </div>
            </>
          )}
        </div>
        
        {/* URLbar */}
        <div className="flex items-center w-full h-10">
          <Urlbar />
        </div>
      </div>
    </div>
  )
}


import type { FC } from 'react'
import { useAtomValue } from 'jotai'
import { sidebarPositionAtom } from '~/atoms/layout'
import { Urlbar } from '../sidebar/urlbar'
import { LeftToolbar, RightToolbar, WindowControlsToolbar } from './toolbar'
import { cn } from '~/lib/helper'
import { useWindowDrag } from '~/hooks/common/use-window-drag'

// Detect operating system
const getOS = (): 'windows' | 'mac' | 'linux' => {
  if (typeof window === 'undefined') return 'windows'
  
  const userAgent = window.navigator.userAgent.toLowerCase()
  if (userAgent.includes('mac')) return 'mac'
  if (userAgent.includes('linux')) return 'linux'
  return 'mac'
}

// Header for multi-toolbar mode: rendered in content navbar area
export const HeaderMulti: FC = () => {
  const rightSide = useAtomValue(sidebarPositionAtom) === 'right'
  const dragRef = useWindowDrag()
  return (
    <div 
      ref={dragRef}
      className="backdrop-blur-sm backdrop-saturate-[120%]" 
      data-variant="multi"
    >
      <div className="h-[38px] block">
        <div
          id="appcontent-navbar-container"
          className={cn(
            "grid items-center h-[38px] gap-3 px-3",
            !rightSide && "grid-cols-[auto_1fr_auto_auto]",
            rightSide && "grid-cols-[auto_auto_1fr_auto]"
          )}
          data-right-side={rightSide || undefined}
        >
          {rightSide ? (
            <>
              {
                getOS() === 'mac' && (
                  <div className="flex items-center" aria-hidden>
                    <WindowControlsToolbar />
                  </div>
                )
              }
              {/* actions slot (optional) */}
              <div className="flex items-center" aria-hidden>
                <RightToolbar />
              </div>
              {/* navigation buttons */}
              <div className="flex items-center" aria-hidden>
                <LeftToolbar />
              </div>
              {/* urlbar */}
              <div className="flex items-center min-w-0">
                <Urlbar />
              </div>
             
            </>
          ) : (
            <>
              {/* navigation buttons */}
              <div className="flex items-center" aria-hidden>
                <LeftToolbar />
              </div>
              {/* urlbar */}
              <div className="flex items-center min-w-0">
                <Urlbar />
              </div>
              {/* actions slot (optional) */}
              <div className="flex items-center" aria-hidden>
                <RightToolbar />
              </div>
              {/* window controls */}
              {
                getOS() !== 'mac' && (
                  <div className="flex items-center" aria-hidden>
                    <WindowControlsToolbar />
                  </div>
                )
              }
            </>
          )}
        </div>
      </div>
    </div>
  )
}


import type { FC } from 'react'
import { useWindowDrag } from '~/hooks/common/use-window-drag'

// Header for compact mode: same placement as multi, reduced density by CSS
export const HeaderCompact: FC = () => {
  const dragRef = useWindowDrag()
  
  return (
    <div 
      ref={dragRef}
      className="bg-transparent" 
      data-variant="compact"
    >
      {/* urlbar slot (compact, in content navbar, denser) */}
    </div>
  )
}


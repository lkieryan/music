import type { FC } from 'react'

// Header for compact mode: same placement as multi, reduced density by CSS
export const HeaderCompact: FC = () => {
  return (
    <div className="bg-transparent" data-variant="compact">
      {/* urlbar slot (compact, in content navbar, denser) */}
    </div>
  )
}


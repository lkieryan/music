import type { FC } from 'react'

export const SidebarPlayer: FC<{ height: number; position?: 'bottom' | 'middle' }> = ({
  height,
  position = 'bottom'
}) => {
  return (
    <div
      className={`w-auto bg-black/20 backdrop-blur-md backdrop-saturate-[120%] border-black/8 z-[2] ${position === 'middle'
        ? 'my-2 mx-1.5 rounded-[8px] border'
        : 'sticky bottom-0'
        }`}
      style={{ height }}
    />
  )
}


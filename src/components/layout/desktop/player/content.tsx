import type { FC } from 'react'

export const ContentPlayer: FC<{ height: number }> = ({ height }) => {
  return (
    <div 
      className="w-full bg-black/20 backdrop-blur-md backdrop-saturate-[120%] border-black/8 sticky bottom-0 z-[2]" 
      style={{ height }} 
    />
  )
}


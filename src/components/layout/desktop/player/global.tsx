import type { FC } from 'react'

export const GlobalPlayer: FC<{ height: number }> = ({ height }) => {
  return (
    <div 
      className="w-full bg-black/20 backdrop-blur-md backdrop-saturate-[120%] border-black/8 fixed left-0 right-0 bottom-0 z-[1000]" 
      style={{ height }} 
    />
  )
}


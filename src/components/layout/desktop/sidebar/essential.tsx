import type { FC } from 'react'
import { useAtomValue } from 'jotai'
import { compactModeAtom, sidebarPositionAtom } from '~/atoms/layout'
import { cn } from '~/lib/helper'

type EssentialStub = {
  id: string
  title: string
  iconUrl: string
  active?: boolean
}

// Static demo data (layout-only)
const mockEssentials: EssentialStub[] = [
  { id: 'github', title: 'GitHub', iconUrl: '/vite.svg' },
  { id: 'discord', title: 'Discord', iconUrl: '/vite.svg', active: true },
  { id: 'notion', title: 'Notion', iconUrl: '/vite.svg' },
  { id: 'calendar', title: 'Calendar', iconUrl: '/vite.svg' },
  { id: 'figma', title: 'Figma', iconUrl: '/vite.svg' },
]

const EssentialItem: FC<{ 
  title: string
  iconUrl: string
  active?: boolean
  isCompact?: boolean
  sidebar?: 'left' | 'right'
}> = ({ title, iconUrl, active = false, isCompact = false, sidebar = 'left' }) => {
  return (
    <div 
      className={cn(
        "grid place-items-center cursor-pointer select-none border border-transparent",
        "transition-all duration-150 ease-out",
        !isCompact && "min-w-[38px] min-h-[48px] rounded-[8px]",
        isCompact && "p-1.5 min-w-[40px] min-h-[40px] rounded-[6px]",
        isCompact && "p-1.5 min-w-[40px] min-h-[40px] rounded-[6px]",
        "bg-[rgba(0,0,0,0.06)] dark:bg-[rgba(255,255,255,0.08)]", 
        "hover:bg-[rgba(0,0,0,0.10)] dark:hover:bg-[rgba(255,255,255,0.18)]",
        active && "border-[rgba(0,0,0,0.08)] dark:border-[rgba(255,255,255,0.12)]",
        "active:scale-[0.98]"
      )}
      style={{
        ...(sidebar === 'right' && {
          direction: 'ltr'
        })
      }}
      title={title}
      aria-selected={active}
      data-active={active || undefined}
    >
      <div 
        className={cn(
          "bg-cover bg-center",
          !isCompact && "w-[18px] h-[18px] rounded-[6px]",
          isCompact && "w-4 h-4 rounded-[4px]"
        )}
        style={{ backgroundImage: `url(${iconUrl})` }}
      />
      <div className="hidden">{title}</div>
    </div>
  )
}

export const EssentialsSection: FC = () => {
  const isCompact = useAtomValue(compactModeAtom)
  const sidebar = useAtomValue(sidebarPositionAtom)

  return (
    <section 
      className={cn(
        "flex flex-col",
        !isCompact && "px-1.5 pt-2 pb-1.5 gap-2",
        isCompact && "px-1 pt-1 pb-0.5 gap-1.5"
      )}
    >
      <div 
        className={cn(
          "grid",
          !isCompact && "gap-1.5",
          isCompact && "gap-1"
        )}
        style={{
          gridTemplateColumns: isCompact 
            ? "repeat(auto-fit, minmax(40px, 1fr))"
            : "repeat(auto-fit, minmax(56px, 1fr))",
             ...(sidebar === 'right' && {
            direction: 'rtl'
          })
        }}
      >
        {mockEssentials.map((item) => (
          <EssentialItem 
            key={item.id} 
            title={item.title} 
            iconUrl={item.iconUrl}
            active={item.active}
            isCompact={isCompact}
            sidebar={sidebar}
          />
        ))}
      </div>
    </section>
  )
}

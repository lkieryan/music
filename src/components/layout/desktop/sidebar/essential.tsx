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
        // 基础样式
        "grid place-items-center cursor-pointer select-none border border-transparent",
        "transition-all duration-150 ease-out",
        // 正常模式
        !isCompact && "min-w-[38px] min-h-[48px] rounded-[8px]",
        // 紧凑模式
        isCompact && "p-1.5 min-w-[40px] min-h-[40px] rounded-[6px]",
        // 背景颜色 - 使用原版的CSS变量值
        "bg-[rgba(0,0,0,0.06)] dark:bg-[rgba(255,255,255,0.08)]",
        // 悬停效果
        "hover:bg-[rgba(0,0,0,0.10)] dark:hover:bg-[rgba(255,255,255,0.18)]",
        // 激活状态
        active && "border-[rgba(0,0,0,0.08)] dark:border-[rgba(255,255,255,0.12)]",
        // 按压效果
        "active:scale-[0.98]"
      )}
      style={{
        // 右侧布局时恢复正常方向，抵消父容器的RTL
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
          // 正常模式图标大小
          !isCompact && "w-[18px] h-[18px] rounded-[6px]",
          // 紧凑模式图标大小
          isCompact && "w-4 h-4 rounded-[4px]"
        )}
        style={{ backgroundImage: `url(${iconUrl})` }}
      />
      {/* 标签始终隐藏 - 和原版一致 */}
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
        // 正常模式：8px 6px 6px padding, 8px gap
        !isCompact && "px-1.5 pt-2 pb-1.5 gap-2",
        // 紧凑模式：4px 4px 3px padding, 6px gap  
        isCompact && "px-1 pt-1 pb-0.5 gap-1.5"
      )}
    >
      <div 
        className={cn(
          "grid",
          // 正常模式：6px gap, 响应式网格 minmax(56px, 1fr)
          !isCompact && "gap-1.5",
          // 紧凑模式：4px gap, 响应式网格 minmax(40px, 1fr)
          isCompact && "gap-1"
        )}
        style={{
          // 使用CSS Grid的minmax实现响应式列数
          gridTemplateColumns: isCompact 
            ? "repeat(auto-fit, minmax(40px, 1fr))"
            : "repeat(auto-fit, minmax(56px, 1fr))",
          // 右侧布局时改变网格的方向，但不影响子元素
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

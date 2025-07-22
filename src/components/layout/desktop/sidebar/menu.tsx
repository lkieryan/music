import type { FC } from 'react'
import { useNavigate } from 'react-router'
import { useAtomValue, useSetAtom } from 'jotai'
import { 
  toolbarDensityAtom, 
  fixedMenusAtom, 
  dynamicMenusAtom,
  activeMenuIdAtom, 
  setActiveMenuAtom,
  type MenuItem as MenuItemType
} from '~/atoms/menu'
import { sidebarPositionAtom } from '~/atoms/layout'
import { cn } from '~/lib/helper'

export const MenuShell: FC = () => {
  const fixedItems = useAtomValue(fixedMenusAtom)
  const dynamicItems = useAtomValue(dynamicMenusAtom)
  const sidebar = useAtomValue(sidebarPositionAtom)
  const density = useAtomValue(toolbarDensityAtom)

  return (
    <aside className="flex flex-col h-screen px-1.5 py-2 gap-2">
      {/* Fixed Section */}
      <div className="flex flex-col gap-1.5" role="menu" aria-label="Fixed">
        <div className={cn(
          "sticky top-0 z-10 bg-inherit px-1 py-1.5 text-xs font-semibold text-text-secondary",
          sidebar === 'right' && "text-right"
        )}>
          Fixed
        </div>
        <div className="flex flex-col gap-1">
          {fixedItems.map((item) => (
            <MenuItem key={item.id} item={item} sidebar={sidebar} density={density} />
          ))}
        </div>
      </div>

      {/* Dynamic Section */}
      <div className="flex flex-col gap-1.5 min-h-0 flex-1" role="menu" aria-label="Recent">
        <div className={cn(
          "sticky top-0 z-10 bg-inherit px-1 py-1.5 text-xs font-semibold text-text-secondary",
          sidebar === 'right' && "text-right"
        )}>
          Recent
        </div>
        <div className="flex flex-col gap-1 min-h-0 flex-1 overflow-y-auto overscroll-contain">
          {dynamicItems.map((item) => (
            <MenuItem key={item.id} item={item} sidebar={sidebar} density={density} />
          ))}
        </div>
      </div>
    </aside>
  )
}

export const MenuItem: FC<{ 
  item: MenuItemType
  sidebar: 'left' | 'right'
  density: 'compact' | 'regular'
}> = ({ item, sidebar, density }) => {
  const activeId = useAtomValue(activeMenuIdAtom)
  const setActive = useSetAtom(setActiveMenuAtom)
  const navigate = useNavigate()

  const active = activeId === item.id

  const onClick = () => {
    setActive(item.id)
    navigate(item.path || '/')
  }

  // 简化：只保留核心功能，删除复杂的data属性逻辑
  const isCompact = density === 'compact'

  return (
    <div
      className={cn(
        // 基础样式 - 使用原版的8px圆角
        "group relative flex items-center gap-2 h-10 px-2.5 rounded-[8px] cursor-pointer select-none",
        "transition-colors duration-150 ease-in-out",
        // 默认文字颜色
        "text-text-secondary",
        // 紧凑模式
        isCompact && "h-8.5 justify-center px-0",
        // 右侧布局时反转flex方向并添加右对齐
        sidebar === 'right' && !isCompact && "flex-row-reverse text-right",
        // 焦点样式
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent focus-visible:outline-offset-2"
      )}
      role="menuitemradio"
      tabIndex={0}
      aria-selected={active}
      aria-current={active ? 'page' : undefined}
      onClick={onClick}
      title={item.label}
    >
      {/* 背景层 - 模拟原来的::before效果，始终存在避免闪烁 */}
      <div 
        className={cn(
          "absolute inset-0.5 rounded-[8px] transition-all duration-100 ease-in-out",
          // 始终有背景，只是透明度和颜色不同
          active 
            ? "bg-[rgba(0,0,0,0.10)] dark:bg-[rgba(255,255,255,0.18)] shadow-[0_1px_1px_rgba(0,0,0,0.08)] dark:shadow-[0_1px_1px_rgba(0,0,0,0.30)]"
            : "bg-transparent group-hover:bg-[rgba(0,0,0,0.06)] dark:group-hover:bg-[rgba(255,255,255,0.08)] shadow-none"
        )}
      />

      {/* 激活指示器 - 简化版本 */}
      {active && (
        <div 
          className={cn(
            "absolute w-0.5 rounded-full bg-current opacity-100 transition-opacity duration-150 z-10",
            "inset-y-0.5",
            sidebar === 'left' ? "left-0.5" : "right-0.5"
          )}
          style={{ color: item.accent || 'currentColor' }}
        />
      )}

      {/* 图标 */}
      <div 
        className={cn(
          "relative z-10 w-4 h-4 bg-cover bg-center rounded-[4px] flex-shrink-0",
          active && "scale-95 transition-transform duration-75"
        )}
        style={{ backgroundImage: item.icon ? `url(${item.icon})` : undefined }}
      />

      {/* 标签 - 紧凑模式下隐藏 */}
      {!isCompact && (
        <div className={cn(
          "relative z-10 text-sm truncate flex-1 transition-colors duration-150",
          active ? "text-text" : "group-hover:text-text"
        )}>
          {item.label}
        </div>
      )}

      {/* 徽章 - 紧凑模式下隐藏 */}
      {!isCompact && item.badge && (
        <div className="relative z-10 text-xs px-1.5 py-0.5 rounded-full bg-text-tertiary/20 text-text-tertiary">
          {item.badge}
        </div>
      )}
    </div>
  )
}

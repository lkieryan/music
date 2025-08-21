import type { FC, SVGProps } from 'react'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useLocation } from 'react-router'
import { useAtomValue, useSetAtom } from 'jotai'
import { 
  toolbarDensityAtom, 
  menusAtom, 
  activeMenuIdAtom, 
  setActiveMenuAtom,
  type MenuItem as MenuItemType
} from '~/atoms/menu'
import { sidebarPositionAtom } from '~/atoms/layout'
import { cn } from '~/lib/helper'
// icons are provided by atoms as React components via item.icon; no local imports needed

export const MenuShell: FC = () => {
  const menus = useAtomValue(menusAtom)
  const sidebar = useAtomValue(sidebarPositionAtom)
  const density = useAtomValue(toolbarDensityAtom)
  const location = useLocation()
  const setActive = useSetAtom(setActiveMenuAtom)

  useEffect(() => {
    const activeMenu = menus.find((item) => location.pathname === item.path || location.pathname.startsWith(item.path + '/'))
    if (activeMenu) {
      setActive(activeMenu.id)
    }
  }, [menus, location, setActive])

  const sortedItems = [...menus].sort((a, b) => {
    const ao = a.order ?? Number.MAX_SAFE_INTEGER
    const bo = b.order ?? Number.MAX_SAFE_INTEGER
    return ao - bo
  })

  return (
    <aside className="flex flex-col px-1.5 py-2 gap-2" role="menu" aria-label="Menu">
      <div className="flex flex-col gap-1 min-h-0 flex-1 overflow-y-auto overscroll-contain">
        {sortedItems.map((item) => (
          <MenuItem key={item.id} item={item} sidebar={sidebar} density={density} />
        ))}
      </div>
    </aside>
  )
}

export const MenuItem: FC<{ 
  item: MenuItemType
  sidebar: 'left' | 'right'
  density: 'compact' | 'regular'
}> = ({ item, sidebar, density }) => {
  const { t } = useTranslation('app')
  const activeId = useAtomValue(activeMenuIdAtom)
  const setActive = useSetAtom(setActiveMenuAtom)
  const navigate = useNavigate()
  const location = useLocation()

  // Derive active from current pathname primarily; fallback to atom when no path
  const active = item.path
    ? (location.pathname === item.path || location.pathname.startsWith(item.path + '/'))
    : (activeId === item.id)

  const onClick = () => {
    setActive(item.id)
    navigate(item.path || '/')
  }

  const isCompact = density === 'compact'

  // Icon rendering: item.icon is expected to be a React component (from atoms)
  // Note: SVGR with { ref: true } exports a ForwardRefExoticComponent (typeof === 'object'),
  // so we must treat any non-string as a component.
  const iconProp = item.icon
  const IconComp: FC<SVGProps<SVGSVGElement>> | undefined =
    iconProp && typeof iconProp !== 'string' ? (iconProp as FC<SVGProps<SVGSVGElement>>) : undefined

  return (
    <div
      className={cn(
        "group relative flex items-center gap-2 h-10 px-2.5 rounded-[8px] cursor-pointer select-none",
        "transition-colors duration-150 ease-in-out",
        "text-text-secondary",
        isCompact && "h-8.5 justify-center px-0",
        sidebar === 'right' && !isCompact && "flex-row-reverse text-right",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent focus-visible:outline-offset-2"
      )}
      role="menuitemradio"
      tabIndex={0}
      aria-selected={active}
      aria-current={active ? 'page' : undefined}
      onClick={onClick}
      title={t(item.label)}
    >
      <div 
        className={cn(
          "absolute inset-0.5 rounded-[8px] transition-all duration-100 ease-in-out",
          active 
            ? "bg-[rgba(0,0,0,0.10)] dark:bg-[rgba(255,255,255,0.18)] shadow-[0_1px_1px_rgba(0,0,0,0.08)] dark:shadow-[0_1px_1px_rgba(0,0,0,0.30)]"
            : "bg-transparent group-hover:bg-[rgba(0,0,0,0.06)] dark:group-hover:bg-[rgba(255,255,255,0.08)] shadow-none"
        )}
      />
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
      {IconComp ? (
        <IconComp className={cn(
          'relative z-10 w-4 h-4 flex-shrink-0',
          active && 'scale-95 transition-transform duration-75'
        )} />
      ) : (
        <div
          className={cn(
            'relative z-10 w-4 h-4 rounded-[4px] flex-shrink-0',
            active && 'scale-95 transition-transform duration-75'
          )}
        />
      )}
      {!isCompact && (
        <div className={cn(
          "relative z-10 text-sm truncate flex-1 transition-colors duration-150",
          active ? "text-text" : "group-hover:text-text"
        )}>
          {t(item.label)}
        </div>
      )}
      {!isCompact && item.badge && (
        <div className="relative z-10 text-xs px-1.5 py-0.5 rounded-full bg-text-tertiary/20 text-text-tertiary">
          {item.badge}
        </div>
      )}
    </div>
  )
}

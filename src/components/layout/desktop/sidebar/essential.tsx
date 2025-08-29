import type { FC } from 'react'
import { useState, useCallback, useRef, useEffect, useMemo } from 'react'
import { useAtomValue } from 'jotai'
import { compactModeAtom, sidebarPositionAtom } from '~/atoms/layout'
import { cn } from '~/lib/helper'
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
  DragOverEvent,
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  rectSortingStrategy,
} from '@dnd-kit/sortable'
import {
  useSortable,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { restrictToParentElement, restrictToFirstScrollableAncestor } from '@dnd-kit/modifiers'

import KieranMusicPng from '~/assets/icons/kieran_music.png'
import { pluginService } from '~/services/plugin-service'
import { resolveImageUrl } from '~/lib/image'
import { useMusicSettingKey, setMusic } from '~/atoms/settings/music'
import { listen } from '@tauri-apps/api/event'

type EssentialStub = {
  id: string
  title: string
  iconUrl: string
}

// Minimal plugin info used for Essentials
type EssentialsPlugin = {
  id: string
  name: string
  display_name: string
  plugin_type: string
  enabled: boolean
  icon?: string | null
}

const EssentialItem: FC<{ 
  id: string
  title: string
  iconUrl: string
  active?: boolean
  isCompact?: boolean
  sidebar?: 'left' | 'right'
  onClick?: (id: string) => void
}> = ({ id, title, iconUrl, active = false, isCompact = false, sidebar = 'left', onClick }) => {
  // Only apply sortable hooks for non-KieranMusic items
  const isDraggable = id !== 'KieranMusic'
  
  const sortableProps = useSortable({
    id: id,
    disabled: !isDraggable,
  })
  
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = isDraggable ? sortableProps : {
    attributes: {},
    listeners: {},
    setNodeRef: () => {},
    transform: null,
    transition: undefined,
    isDragging: false,
  }

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }
  const handleClick = useCallback(() => {
    onClick?.(id)
  }, [id, onClick])
  
  return (
    <div 
      ref={isDraggable ? setNodeRef : undefined}
      style={isDraggable ? style : undefined}
      className={cn(
        "relative grid place-items-center cursor-pointer select-none",
        "transition-all duration-200 ease-out",
        !isCompact && "min-w-[38px] min-h-[48px] rounded-[10px]",
        isCompact && "min-w-[40px] min-h-[40px] rounded-[8px]",
        "bg-[rgba(0,0,0,0.03)] dark:bg-[rgba(255,255,255,0.05)]",
        "hover:bg-[rgba(0,0,0,0.06)] dark:hover:bg-[rgba(255,255,255,0.08)]",
        "active:scale-[0.96]",
        "backdrop-blur-sm",
        active && "overflow-hidden !bg-transparent",
        // Drag state styles only for draggable items
        isDraggable && "cursor-grab active:cursor-grabbing",
        isDragging && "opacity-50 z-50",
        !isDraggable && "cursor-pointer"
      )}
      {...(isDraggable ? { ...attributes, ...listeners } : {})}
      onClick={handleClick}
      title={title}
      aria-selected={active}
      data-active={active || undefined}
      data-essential-id={id}
    >
      {active && (
        <>
          <div 
            className="absolute z-[-1]"
            style={{
              inset: '-50%',
              filter: 'blur(15px)',
              backgroundImage: `url(${iconUrl})`,
              backgroundSize: 'contain',
              backgroundPosition: 'center',
              backgroundClip: 'padding-box',
              opacity: '0.6',
            }}
          />
          <div 
            className={cn(
              "absolute z-[0] transition-all duration-100 ease-in-out backdrop-blur-sm",
              "bg-white/65 dark:bg-gray-700/65"
            )}
            style={{
              inset: '0',
              margin: '2px',
              borderRadius: 'calc(10px - 2px)',
            }}
          />
        </>
      )}
      
      <div 
        className={cn(
          "relative flex items-center justify-center z-10",
          !isCompact && "w-[18px] h-[18px]",
          isCompact && "w-4 h-4"
        )}
      >
        <img 
          src={iconUrl}
          alt={title}
          className={cn(
            "w-full h-full object-contain transition-opacity duration-200",
            active ? "opacity-100" : "opacity-80 hover:opacity-90"
          )}
        />
      </div>
      
      <div className="sr-only">{title}</div>
    </div>
  )
}

export const EssentialsSection: FC = () => {
  const isCompact = useAtomValue(compactModeAtom)
  const sidebar = useAtomValue(sidebarPositionAtom)
  const [selectedId, setSelectedId] = useState<string>('KieranMusic')
  const [essentials, setEssentials] = useState<EssentialStub[]>([])
  const [plugins, setPlugins] = useState<EssentialsPlugin[]>([])
  const source = useMusicSettingKey('source')
  const sourcesOrder = useMusicSettingKey('sourcesOrder')
  const [containerWidth, setContainerWidth] = useState(0)
  const containerRef = useRef<HTMLDivElement>(null)
  
  // Separate essential item from draggable items
  const essentialItem = essentials.find(item => item.id === 'KieranMusic')
  const draggableItems = essentials.filter(item => item.id !== 'KieranMusic')

  // Derive selectedId from music.source
  useEffect(() => {
    if (!source) return
    if (source.mode === 'all' || !source.ids || source.ids.length === 0) {
      setSelectedId('KieranMusic')
    } else if (source.mode === 'single') {
      setSelectedId(source.ids[0] ?? 'KieranMusic')
    } else if (source.mode === 'many') {
      // Many mode not surfaced in UI yet; fallback highlight first id
      setSelectedId(source.ids[0] ?? 'KieranMusic')
    }
  }, [source])

  // Fetch plugins (enabled audio providers only)
  const refreshPlugins = useCallback(async () => {
    try {
      const list = await pluginService.getPlugins()
      const filtered = list.filter((p: any) => {
        const t = String(p.plugin_type || '')
          .toLowerCase()
          .replace(/[-_\s]/g, '')
        // Only accept AudioProvider, exclude AudioProcessor, etc. (robust)
        const isAudio = t === 'audioprovider'
        return isAudio && !!p.enabled
      }) as EssentialsPlugin[]
      setPlugins(filtered)
    } catch (e) {
      console.error('[Essentials] Failed to load plugins', e)
      setPlugins([])
    }
  }, [])

  // Throttle refresh requests triggered by events to avoid storms
  const refreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const scheduleRefresh = useCallback(() => {
    if (refreshTimerRef.current) {
      clearTimeout(refreshTimerRef.current)
      refreshTimerRef.current = null
    }
    refreshTimerRef.current = setTimeout(() => {
      refreshPlugins()
      if (refreshTimerRef.current) {
        clearTimeout(refreshTimerRef.current)
        refreshTimerRef.current = null
      }
    }, 200)
  }, [refreshPlugins])

  useEffect(() => {
    let mounted = true
    refreshPlugins()
    // Listen backend plugin updates
    let unlisten: (() => void) | undefined
    listen('plugins-updated', () => {
      if (!mounted) return
      scheduleRefresh()
    }).then((fn) => { unlisten = fn as any }).catch(() => {})
    // Also refresh when window regains focus (covers cross-window/state drift)
    let unlistenFocus: (() => void) | undefined
    listen('tauri://focus', () => {
      if (!mounted) return
      scheduleRefresh()
    }).then((fn) => { unlistenFocus = fn as any }).catch(() => {})
    return () => {
      mounted = false
      if (unlisten) unlisten()
      if (unlistenFocus) unlistenFocus()
      if (refreshTimerRef.current) {
        clearTimeout(refreshTimerRef.current)
        refreshTimerRef.current = null
      }
    }
  }, [refreshPlugins, scheduleRefresh])

  // Compute ordered items from plugins + sourcesOrder
  const orderedPluginItems: EssentialStub[] = useMemo(() => {
    const order = Array.isArray(sourcesOrder) ? sourcesOrder : []
    const map = new Map(plugins.map(p => [p.id, p]))
    const pickIcon = (p: EssentialsPlugin): string => {
      const url = resolveImageUrl(p.icon ?? null)
      if (url) return url
      // Fallback to app icon when plugin does not provide an icon
      return KieranMusicPng
    }
    const fromOrder: EssentialStub[] = order
      .map(id => map.get(id))
      .filter(Boolean)
      .map((p: any) => ({ id: p.id, title: p.display_name || p.name, iconUrl: pickIcon(p) }))
    const remaining: EssentialStub[] = plugins
      .filter(p => !order.includes(p.id))
      .map(p => ({ id: p.id, title: p.display_name || p.name, iconUrl: pickIcon(p) }))
    return [...fromOrder, ...remaining]
  }, [plugins, sourcesOrder])

  // Build essentials array: KieranMusic + ordered plugins
  useEffect(() => {
    const items: EssentialStub[] = [
      { id: 'KieranMusic', title: 'KieranMusic', iconUrl: KieranMusicPng },
      ...orderedPluginItems,
    ]
    setEssentials(items)
  }, [orderedPluginItems])

  // If selected source becomes invalid (plugin disabled/removed), fallback to KieranMusic
  useEffect(() => {
    if (!source) return
    if (source.mode === 'single') {
      const id = source.ids?.[0]
      if (id && !plugins.some(p => p.id === id)) {
        setMusic('source', { mode: 'all', ids: [] })
      }
    } else if (source.mode === 'many') {
      const ids = Array.isArray(source.ids) ? source.ids : []
      const valid = ids.filter(id => plugins.some(p => p.id === id))
      if (valid.length !== ids.length) {
        if (valid.length === 0) setMusic('source', { mode: 'all', ids: [] })
        else setMusic('source', { mode: 'many', ids: valid })
      }
    }
  }, [plugins, source])

  // Clean invalid ids from sourcesOrder when plugin set changes
  useEffect(() => {
    const order = Array.isArray(sourcesOrder) ? sourcesOrder : []
    if (order.length === 0) return
    const validIds = new Set(plugins.map(p => p.id))
    const cleaned = order.filter((id: string) => validIds.has(id))
    if (cleaned.length !== order.length) {
      setMusic('sourcesOrder', cleaned)
    }
  }, [plugins, sourcesOrder])

  // Effect to observe container width changes
  useEffect(() => {
    const container = containerRef.current
    if (!container) return
    
    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerWidth(entry.contentRect.width)
      }
    })
    
    resizeObserver.observe(container)
    
    // Initial measurement
    setContainerWidth(container.getBoundingClientRect().width)
    
    return () => {
      resizeObserver.disconnect()
    }
  }, [])
  
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // Require 8px movement before drag starts
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  )
  
  const handleItemClick = useCallback((id: string) => {
    setSelectedId(id)
    if (id === 'KieranMusic') {
      setMusic('source', { mode: 'all', ids: [] })
    } else {
      setMusic('source', { mode: 'single', ids: [id] })
    }
  }, [])
  
  const handleDragOver = useCallback((_event: DragOverEvent) => {
    // No state mutation during drag to avoid jitter. Visual movement is handled by dnd-kit transforms.
  }, [])
  
  const handleDragEnd = useCallback((event: DragEndEvent) => {
    const { active, over } = event
    if (!over || active.id === over.id) return
    // Build current plugin id order from rendered items (excluding KieranMusic)
    const ids = orderedPluginItems.map(i => i.id)
    const from = ids.indexOf(String(active.id))
    const to = ids.indexOf(String(over.id))
    if (from === -1 || to === -1) return
    const next = arrayMove(ids, from, to)
    setMusic('sourcesOrder', next)
  }, [orderedPluginItems])

  return (
    <section 
      className={cn(
        "flex flex-col zen-essentials-container",
        !isCompact && "px-1.5 pt-2 pb-1.5 gap-2",
        isCompact && "px-1 pt-1 pb-0.5 gap-1.5",
        // Prevent horizontal scrollbar during drag
        "overflow-x-hidden"
      )}
      data-zen-essential="true"
    >
      <DndContext 
        sensors={sensors}
        collisionDetection={closestCenter}
        modifiers={[restrictToParentElement, restrictToFirstScrollableAncestor]}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
      >
        <SortableContext 
          items={draggableItems.map(item => item.id)}
          strategy={rectSortingStrategy}
        >
          <div 
            ref={containerRef}
            className={cn(
              "grid transition-all duration-300 ease-out overflow-x-hidden",
              !isCompact && "gap-1.5",
              isCompact && "gap-1"
            )}
            style={{
              gridTemplateColumns: containerWidth < 120 ? '1fr' :
                                 containerWidth < 160 ? 'repeat(2, 1fr)' :
                                 containerWidth < 200 ? 'repeat(3, 1fr)' : 
                                 'repeat(4, 1fr)',
              ...(sidebar === 'right' && {
                direction: 'rtl'
              })
            }}
          >
            {/* essential item - fixed position, not draggable */}
            {essentialItem && (
              <EssentialItem 
                key={essentialItem.id}
                id={essentialItem.id}
                title={essentialItem.title}
                iconUrl={essentialItem.iconUrl}
                active={selectedId === essentialItem.id}
                isCompact={isCompact}
                sidebar={sidebar}
                onClick={handleItemClick}
              />
            )}
            
            {/* Draggable items */}
            {draggableItems.map((item) => (
              <EssentialItem 
                key={item.id}
                id={item.id}
                title={item.title}
                iconUrl={item.iconUrl}
                active={selectedId === item.id}
                isCompact={isCompact}
                sidebar={sidebar}
                onClick={handleItemClick}
              />
            ))}
          </div>
        </SortableContext>
      </DndContext>
    </section>
  )
}

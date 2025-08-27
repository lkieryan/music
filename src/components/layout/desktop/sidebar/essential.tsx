import type { FC } from 'react'
import { useState, useCallback, useRef, useEffect } from 'react'
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

import SpotifyPng from '~/assets/icons/spotify.png'
import YouTubePng from '~/assets/icons/youtube.png'
import BilibiliPng from '~/assets/icons/bilibili.png'
import QQMusicPng from '~/assets/icons/qq_music.png'
import KugouPng from '~/assets/icons/kugou.png'
import KieranMusicPng from '~/assets/icons/kieran_music.png'
import Netease from '~/assets/icons/netease.png'

type EssentialStub = {
  id: string
  title: string
  iconUrl: string
}


const mockEssentials: EssentialStub[] = [
  { 
    id: 'home', 
    title: 'Home', 
    iconUrl: KieranMusicPng,
  },
  { 
    id: 'spotify', 
    title: 'Spotify', 
    iconUrl: SpotifyPng,
  },
  { 
    id: 'youtube', 
    title: 'YouTube', 
    iconUrl: YouTubePng,
  },
  { 
    id: 'bilibili', 
    title: 'Bilibili', 
    iconUrl: BilibiliPng,
  },
  { 
    id: 'qq-music', 
    title: 'QQ Music', 
    iconUrl: QQMusicPng,
  },
  { 
    id: 'kugou', 
    title: 'Kugou', 
    iconUrl: KugouPng,
  },
  { 
    id: 'netease', 
    title: 'Netease', 
    iconUrl: Netease,
  },
]

const EssentialItem: FC<{ 
  id: string
  title: string
  iconUrl: string
  active?: boolean
  isCompact?: boolean
  sidebar?: 'left' | 'right'
  onClick?: (id: string) => void
}> = ({ id, title, iconUrl, active = false, isCompact = false, sidebar = 'left', onClick }) => {
  // Only apply sortable hooks for non-home items
  const isDraggable = id !== 'home'
  
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
  const [selectedId, setSelectedId] = useState<string>('spotify')
  const [essentials, setEssentials] = useState(mockEssentials)
  const [containerWidth, setContainerWidth] = useState(0)
  const containerRef = useRef<HTMLDivElement>(null)
  
  // Separate home item from draggable items
  const homeItem = essentials.find(item => item.id === 'home')
  const draggableItems = essentials.filter(item => item.id !== 'home')
  
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
  }, [])
  
  const handleDragStart = useCallback((event: { active: { id: any } }) => {
    // Future: Could be used for drag preview or state tracking
  }, [])
  
  const handleDragOver = useCallback((event: DragOverEvent) => {
    const { active, over } = event
    
    if (active.id !== over?.id) {
      const newDraggableItems = [...draggableItems]
      const oldIndex = newDraggableItems.findIndex((item) => item.id === active.id)
      const newIndex = newDraggableItems.findIndex((item) => item.id === over?.id)
      
      if (oldIndex !== -1 && newIndex !== -1) {
        const reorderedItems = arrayMove(newDraggableItems, oldIndex, newIndex)
        // Combine home item with reordered draggable items
        setEssentials([homeItem!, ...reorderedItems])
      }
    }
  }, [draggableItems, homeItem])
  
  const handleDragEnd = useCallback((event: DragEndEvent) => {
    // Future: Could be used for cleanup or final state updates
  }, [])

  return (
    <section 
      className={cn(
        "flex flex-col zen-essentials-container",
        !isCompact && "px-1.5 pt-2 pb-1.5 gap-2",
        isCompact && "px-1 pt-1 pb-0.5 gap-1.5"
      )}
      data-zen-essential="true"
    >
      <DndContext 
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragStart={handleDragStart}
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
              "grid transition-all duration-300 ease-out",
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
            {/* Home item - fixed position, not draggable */}
            {homeItem && (
              <EssentialItem 
                key={homeItem.id}
                id={homeItem.id}
                title={homeItem.title}
                iconUrl={homeItem.iconUrl}
                active={selectedId === homeItem.id}
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
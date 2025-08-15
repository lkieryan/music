import type { FC, PropsWithChildren, MouseEvent } from 'react'
import { useCallback, useLayoutEffect, useRef } from 'react'
import { useAtom, useAtomValue } from 'jotai'
import { compactModeAtom, sidebarMaxWidth, sidebarMinWidth, sidebarWidthAtom } from '~/atoms/layout'

export const SidebarBox: FC<PropsWithChildren<{ positionEnd?: boolean }>> = ({ positionEnd, children }) => {
  const [width, setWidth] = useAtom(sidebarWidthAtom)
  const isCompact = useAtomValue(compactModeAtom)
  const ref = useRef<HTMLElement | null>(null)

  const computeCols = (w: number): 2 | 3 | 4 => (w >= 260 ? 4 : w >= 200 ? 3 : 2)
  const applyColsAttr = useCallback((el: HTMLElement, w: number) => {
    el.setAttribute('data-essentials-cols', String(computeCols(w)))
  }, [])

  // Initialize/update columns on mount and when width/compact changes
  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return
    const w = el.getBoundingClientRect().width
    applyColsAttr(el, w)
  }, [width, isCompact, applyColsAttr])

  const onMouseDown = (e: MouseEvent<HTMLElement>) => {
    if (isCompact) return
    const target = e.currentTarget as HTMLElement
    const rect = target.getBoundingClientRect()
    const edgeSize = 10
    const onLeftEdge = e.clientX - rect.left <= edgeSize
    const onRightEdge = rect.right - e.clientX <= edgeSize
    const isEdge = positionEnd ? onLeftEdge : onRightEdge
    if (!isEdge) return

    e.preventDefault()
    const startX = e.clientX
    const startWidth = rect.width
    const dir = positionEnd ? -1 : 1

    const onMove = (ev: globalThis.MouseEvent) => {
      const delta = ev.clientX - startX
      let next = startWidth + dir * delta
      next = Math.max(sidebarMinWidth, Math.min(sidebarMaxWidth, next))
      setWidth(next)
      document.documentElement.style.setProperty('--sidebar-width', `${next}px`)
      applyColsAttr(target, next)
    }
    const onUp = () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    document.body.style.cursor = 'ew-resize'
    document.body.style.userSelect = 'none'
  }

  const onMouseMove = (e: MouseEvent<HTMLElement>) => {
    if (isCompact) return
    const target = e.currentTarget as HTMLElement
    const rect = target.getBoundingClientRect()
    const edgeSize = 10
    const onLeftEdge = e.clientX - rect.left <= edgeSize
    const onRightEdge = rect.right - e.clientX <= edgeSize
    const isEdge = positionEnd ? onLeftEdge : onRightEdge
    target.style.cursor = isEdge ? 'ew-resize' : ''
  }

  const onMouseLeave = (e: MouseEvent<HTMLElement>) => {
    const target = e.currentTarget as HTMLElement
    target.style.cursor = ''
  }

  return (
    <aside
      className="bg-transparent backdrop-blur-sm backdrop-saturate-[120%] max-h-screen overflow-x-hidden relative"
      data-position-end={positionEnd || undefined}
      style={
        !isCompact 
          ? ({ width } as React.CSSProperties) 
          : ({ width: '56px' } as React.CSSProperties)
      }
      onMouseDown={onMouseDown}
      onMouseMove={onMouseMove}
      onMouseLeave={onMouseLeave}
      ref={(el) => {
        ref.current = el
      }}
    >
      {children}
    </aside>
  )
}



import { Spring } from "~/constants/spring"
import { LetsIconsResizeDownRightLight } from "~/components/icons/resize"
import { preventDefault } from "~/lib/dom"
import { cn } from "~/lib/helper"
import type { BoundingBox } from "motion/react"
import { Resizable } from "re-resizable"
import type { PropsWithChildren } from "react"
import { memo, Suspense, useCallback, useEffect, useRef } from "react"

import { useUISettingSelector } from "~/atoms/settings/ui"
import { m } from "~/components/common/motion"
import { resizableOnly } from "~/components/ui/modal"
import { useModalResizeAndDrag } from "~/components/ui/modal/stacked/internal/use-drag"

import { SETTING_MODAL_ID } from "../constants"
import { EnhancedSettingsIndicator } from "../helper/EnhancedIndicator"
import { useAvailableSettings, useSettingPageContext } from "../hooks/use-setting-ctx"
import { SettingsSidebarTitle } from "../title"
import type { SettingPageConfig } from "../utils"
import { DisableWhy } from "../utils"
import { useSetSettingTab, useSettingTab } from "./context"
import { defaultCtx, SettingContext } from "./hooks"

export function SettingModalLayout(
  props: PropsWithChildren<{
    initialTab?: string
  }>,
) {
  const { children, initialTab } = props
  const setTab = useSetSettingTab()
  const tab = useSettingTab()
  const elementRef = useRef<HTMLDivElement>(null)
  const edgeElementRef = useRef<HTMLDivElement>(null)
  const {
    handleDrag,
    handleResizeStart,
    handleResizeStop,
    preferDragDir,
    isResizeable,
    resizeableStyle,
    dragController,
  } = useModalResizeAndDrag(elementRef, {
    resizeable: true,
    draggable: true,
  })

  const availableSettings = useAvailableSettings()
  useEffect(() => {
    if (!tab) {
      if (initialTab) {
        setTab(initialTab)
      } else {
        setTab(availableSettings[0]!.path)
      }
    }
  }, [availableSettings])

  const { draggable, overlay } = useUISettingSelector((state) => ({
    draggable: state.modalDraggable,
    overlay: state.modalOverlay,
  }))

  const measureDragConstraints = useRef((constraints: BoundingBox) => {
    return constraints
  }).current

  return (
    <div
      id={SETTING_MODAL_ID}
      className={cn("h-full", !isResizeable && "center")}
      ref={edgeElementRef}
    >
      <m.div
        exit={{
          opacity: 0,
          scale: 0.96,
        }}
        transition={Spring.presets.smooth}
        className={cn(
          "relative flex overflow-hidden rounded-xl rounded-br-none border",
          !overlay && "shadow-perfect",
        )}
        style={resizeableStyle}
        onContextMenu={preventDefault}
        drag={draggable && (preferDragDir || draggable)}
        dragControls={dragController}
        dragListener={false}
        dragMomentum={false}
        dragElastic={false}
        dragConstraints={edgeElementRef}
        onMeasureDragConstraints={measureDragConstraints}
        whileDrag={{
          cursor: "grabbing",
        }}
      >
        <SettingContext.Provider value={defaultCtx}>
          <Resizable
            onResizeStart={handleResizeStart}
            onResizeStop={handleResizeStop}
            enable={resizableOnly("bottomRight")}
            defaultSize={{
              width: 950,
              height: 800,
            }}
            maxHeight="90vh"
            minHeight={400}
            minWidth={700}
            maxWidth="95vw"
            className="flex !select-none flex-col"
          >
            {draggable && (
              <div className="absolute inset-x-0 top-0 z-[1] h-8" onPointerDown={handleDrag} />
            )}
            <div className="flex h-0 flex-1" ref={elementRef}>
              <div className="backdrop-blur-sm bg-background/60 border-r border border-border flex min-h-0 min-w-44 max-w-[20ch] flex-col rounded-l-xl px-2 py-6">
                <div className="font-default mb-4 flex h-8 items-center gap-2 px-2 font-bold">
                 logo
                </div>
                <nav className="flex grow flex-col">
                  <SidebarItems />
                </nav>

                <div className="relative -mb-6 flex h-8 shrink-0 items-center justify-end gap-2">
                  <EnhancedSettingsIndicator />
                </div>
              </div>
              <div className="bg-background relative flex h-full min-w-0 flex-1 flex-col pt-1">
                <Suspense>{children}</Suspense>
              </div>
            </div>

            <LetsIconsResizeDownRightLight className="text-border pointer-events-none absolute bottom-0 right-0 size-6 translate-x-px translate-y-px" />
          </Resizable>
        </SettingContext.Provider>
      </m.div>
    </div>
  )
}

const SettingItemButtonImpl = (props: {
  setTab: (tab: string) => void
  item: SettingPageConfig
  path: string
  isActive: boolean
  onChange?: (tab: string) => void
}) => {
  const { setTab, item, path, onChange, isActive } = props
  const { disableIf } = item

  const ctx = useSettingPageContext()

  const [disabled, why] = disableIf?.(ctx) || [false, DisableWhy.Noop]

  return (
    <button
      className={cn(
        "text-text my-0.5 flex w-full items-center rounded-lg px-2.5 py-0.5 leading-loose",
        isActive && "!bg-theme-item-active !text-text",
        "hover:bg-theme-item-hover duration-200",
        disabled && "cursor-not-allowed opacity-50",
      )}
      type="button"
      onClick={useCallback(() => {
        if (disabled) {
          switch (why) {
            case DisableWhy.NotActivation: {
              return
            }
            case DisableWhy.Noop: {
              break
            }
          }
        }
        setTab(path)
        onChange?.(path)
      }, [disabled, why, setTab, path, onChange])}
    >
      <SettingsSidebarTitle path={path} className="text-[0.94rem] font-medium" />
    </button>
  )
}

const SettingItemButton = memo(SettingItemButtonImpl)

export const SidebarItems = memo((props: { onChange?: (tab: string) => void }) => {
  const { onChange } = props
  const setTab = useSetSettingTab()
  const tab = useSettingTab()
  const availableSettings = useAvailableSettings()
  return availableSettings.map((t) => {
    const isActive = tab === t.path
    return (
      <SettingItemButton
        key={t.path}
        isActive={isActive}
        setTab={setTab}
        item={t}
        path={t.path}
        onChange={onChange}
      />
    )
  })
})

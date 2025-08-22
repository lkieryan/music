import { ScrollArea } from "~/components/ui/scroll-area"
import { cn } from "~/lib/helper"
import type { FC } from "react"
import { Suspense, useDeferredValue, useLayoutEffect, useState } from "react"
import { useLoaderData } from "react-router"

import { ModalClose } from "~/components/ui/modal/stacked/components"
import { SettingsTitle } from "~/components/modules/settings/title"

import { getSettingPages } from "../settings-glob"
import type { SettingPageConfig } from "../utils"
import { SettingTabProvider, useSettingTab } from "./context"
import { SettingModalLayout } from "./layout"

export const SettingModalContent: FC<{
  initialTab?: string
}> = ({ initialTab }) => {
  const pages = getSettingPages()
  return (
    <SettingTabProvider>
      <SettingModalLayout
        initialTab={initialTab ? (initialTab in pages ? initialTab : undefined) : undefined}
      >
        <Content />
      </SettingModalLayout>
    </SettingTabProvider>
  )
}

const Content = () => {
  const key = useDeferredValue(useSettingTab() || "general")
  const pages = getSettingPages()
  const { Component, loader } = pages[key]

  const [scroller, setScroller] = useState<HTMLDivElement | null>(null)

  useLayoutEffect(() => {
    if (scroller) {
      scroller.scrollTop = 0
    }
  }, [key])

  const config = (useLoaderData() || loader || {}) as SettingPageConfig
  if (!Component) return null

  return (
    <Suspense>
      <SettingsTitle loader={ loader} className="relative mb-0 px-8" />
      <ModalClose />
      <ScrollArea.ScrollArea
        mask={false}
        ref={setScroller}
        rootClassName="h-full grow flex-1 shrink-0 overflow-auto pl-8 pr-7 pb-"
        viewportClassName={cn(
          "px-1 min-h-full [&>div]:min-h-full [&>div]:relative",
          config.viewportClassName,
        )}
      >
        <Component />
      </ScrollArea.ScrollArea>
    </Suspense>
  )
}

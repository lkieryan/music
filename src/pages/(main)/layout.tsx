import { AppLayout } from '~/components/layout'
import { useAtom } from 'jotai'
import { sidebarPositionAtom, toolbarModeAtom } from '~/atoms/layout'

export function Component() {
  // mount atoms to ensure Provider is aware; values are persisted
  useAtom(toolbarModeAtom)
  useAtom(sidebarPositionAtom)
  return <AppLayout />
}
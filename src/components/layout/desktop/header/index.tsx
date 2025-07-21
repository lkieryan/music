import type { FC } from 'react'
import { useDesktopLayout } from '~/providers/layout-provider'
import { HeaderSingle } from './single'
import { HeaderMulti } from './multi'
import { HeaderCompact } from './compact'

export const Header: FC = () => {
  const { compactMode, singleToolbar } = useDesktopLayout()
  if (singleToolbar) return <HeaderSingle />
  return compactMode ? <HeaderCompact /> : <HeaderMulti />
}


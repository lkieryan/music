import type { FC } from 'react'
import GradientBackground from './gradient'
import DynamicCoverBackground from './dynamic-cover'
import { useBackgroundModeValue } from "~/atoms/settings/themes"

/**
 * Background (central hub)
 * Choose background variant based on app background mode.
 */
export const Background: FC = () => {
  const mode = useBackgroundModeValue()
  if (mode === 'dynamic_cover') return <DynamicCoverBackground />
  return <GradientBackground />
}
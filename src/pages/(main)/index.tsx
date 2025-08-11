
// import type { LayoutShellProps } from '~/components/layout/app/layout-shell'

export function Component() {
  // const [isGradientOpen, setIsGradientOpen] = useState(false)
  // const [currentGradient, setCurrentGradient] = useState<GradientState | null>(null)

  // // layout controls -> write to global atoms (affects outer layout)
  // const [toolbarMode, setToolbarMode] = useAtom(toolbarModeAtom)
  // const [sidebarPosition, setSidebarPosition] = useAtom(sidebarPositionAtom)
  // const [playerPlacement, setPlayerPlacement] = useAtom(playerPlacementAtom)
  // const [playerVisible, setPlayerVisible] = useAtom(playerVisibleAtom)
  // const [playerHeight, setPlayerHeight] = useAtom(playerHeightAtom)

  // const handleGradientChange = useCallback((data: GradientState) => {
  //   setCurrentGradient(data)
    
  //   // Apply the gradient to CSS variables
  //   // Crossfade: move previous background to *-old and reset opacity for fade
  //   const previousBg = getComputedStyle(document.documentElement).getPropertyValue('--main-browser-background')
  //   if (previousBg) {
  //     document.documentElement.style.setProperty('--main-browser-background-old', previousBg.trim())
  //   }

  //   // Set new gradients
  //   document.documentElement.style.setProperty('--main-browser-background', data.radial)
  //   document.documentElement.style.setProperty('--main-browser-background-toolbar', data.linear)

  //   // Opacity transition from previous to 1
  //   const prevOpacity = getComputedStyle(document.documentElement).getPropertyValue('--background-opacity')
  //   let startOpacity = 1
  //   if (prevOpacity) {
  //     const parsed = parseFloat(prevOpacity)
  //     if (!Number.isNaN(parsed)) startOpacity = parsed >= 1 ? 0 : 1 - parsed
  //   }
  //   document.documentElement.style.setProperty('--background-opacity', `${startOpacity}`)
  //   requestAnimationFrame(() => {
  //     document.documentElement.style.setProperty('--background-opacity', `${data.opacity}`)
  //   })

  //   // Grain overlay
  //   document.documentElement.style.setProperty('--grainy-background-opacity', data.texture.toString())
  //   document.documentElement.setAttribute('show-grainy-background', data.texture > 0 ? 'true' : 'false')

  //   // Dark mode + toolbox text color
  //   if (data.toolboxTextColor) {
  //     document.documentElement.style.setProperty('--toolbox-textcolor', data.toolboxTextColor)
  //   }
  //   if (typeof data.shouldBeDark === 'boolean') {
  //     document.documentElement.setAttribute('should-be-dark-mode', data.shouldBeDark ? 'true' : 'false')
  //   }
  // }, [])

  return (
    <div className="min-h-scree">
      home
    </div>
  )
}
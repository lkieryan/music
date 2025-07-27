import { useState, useCallback } from 'react'
import { useAtom } from 'jotai'
import { sidebarPositionAtom, toolbarModeAtom, playerPlacementAtom, playerVisibleAtom, playerHeightAtom } from '~/atoms/layout'
import type { PlayerPlacement } from '~/atoms/layout'
import GradientGeneratorDialog  from '~/components/gradient/index.tsx'
import type { GradientState } from '~/types/gradient'
// import type { LayoutShellProps } from '~/components/layout/app/layout-shell'

export function Component() {
  const [isGradientOpen, setIsGradientOpen] = useState(false)
  const [currentGradient, setCurrentGradient] = useState<GradientState | null>(null)

  // layout controls -> write to global atoms (affects outer layout)
  const [toolbarMode, setToolbarMode] = useAtom(toolbarModeAtom)
  const [sidebarPosition, setSidebarPosition] = useAtom(sidebarPositionAtom)
  const [playerPlacement, setPlayerPlacement] = useAtom(playerPlacementAtom)
  const [playerVisible, setPlayerVisible] = useAtom(playerVisibleAtom)
  const [playerHeight, setPlayerHeight] = useAtom(playerHeightAtom)

  const handleGradientChange = useCallback((data: GradientState) => {
    setCurrentGradient(data)
    
    // Apply the gradient to CSS variables
    // Crossfade: move previous background to *-old and reset opacity for fade
    const previousBg = getComputedStyle(document.documentElement).getPropertyValue('--main-browser-background')
    if (previousBg) {
      document.documentElement.style.setProperty('--main-browser-background-old', previousBg.trim())
    }

    // Set new gradients
    document.documentElement.style.setProperty('--main-browser-background', data.radial)
    document.documentElement.style.setProperty('--main-browser-background-toolbar', data.linear)

    // Opacity transition from previous to 1
    const prevOpacity = getComputedStyle(document.documentElement).getPropertyValue('--background-opacity')
    let startOpacity = 1
    if (prevOpacity) {
      const parsed = parseFloat(prevOpacity)
      if (!Number.isNaN(parsed)) startOpacity = parsed >= 1 ? 0 : 1 - parsed
    }
    document.documentElement.style.setProperty('--background-opacity', `${startOpacity}`)
    requestAnimationFrame(() => {
      document.documentElement.style.setProperty('--background-opacity', `${data.opacity}`)
    })

    // Grain overlay
    document.documentElement.style.setProperty('--grainy-background-opacity', data.texture.toString())
    document.documentElement.setAttribute('show-grainy-background', data.texture > 0 ? 'true' : 'false')

    // Dark mode + toolbox text color
    if (data.toolboxTextColor) {
      document.documentElement.style.setProperty('--toolbox-textcolor', data.toolboxTextColor)
    }
    if (typeof data.shouldBeDark === 'boolean') {
      document.documentElement.setAttribute('should-be-dark-mode', data.shouldBeDark ? 'true' : 'false')
    }
  }, [])

  return (
    <div className="min-h-scree">
      {/* layout control panel */}
      <div>
        <label>
          Toolbar:
          <select value={toolbarMode} onChange={(e) => setToolbarMode(e.target.value as 'single' | 'multi' | 'compact')}>
            <option value="single">Single Toolbar</option>
            <option value="multi">Multi Toolbar</option>
            <option value="compact">Compact</option>
          </select>
        </label>
        <label>
          Sidebar:
          <select value={sidebarPosition} onChange={(e) => setSidebarPosition(e.target.value as 'left' | 'right')}>
            <option value="left">Left</option>
            <option value="right">Right</option>
          </select>
        </label>

        {/* Player layout controls */}
        <label style={{ marginLeft: 12 }}>
          Player:
          <select value={playerPlacement} onChange={(e) => setPlayerPlacement(e.target.value as PlayerPlacement)}>
            <option value="none">None</option>
            <option value="global-bottom">Global Bottom</option>
            <option value="content-bottom">Content Bottom</option>
            <option value="sidebar">Sidebar</option>
          </select>
        </label>
        <label style={{ marginLeft: 8 }}>
          Visible:
          <input type="checkbox" checked={playerVisible} onChange={(e) => setPlayerVisible(e.target.checked)} />
        </label>
        <label style={{ marginLeft: 8 }}>
          Height:
          <input
            type="number"
            min={40}
            max={160}
            value={playerHeight}
            onChange={(e) => setPlayerHeight(Number.parseInt(e.target.value || '0', 10) || 0)}
            style={{ width: 64 }}
          />
        </label>
      </div>
      <div >
        <h1 className="text-3xl font-bold text-gray-900 mb-8">
           Gradient Generator Demo
        </h1>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          {/* Controls */}
          <div className="bg-white rounded-lg shadow-sm p-6">
            <h2 className="text-xl font-semibold mb-4">Controls</h2>
            
            <button
              onClick={() => setIsGradientOpen(true)}
              className="w-full bg-blue-500 hover:bg-blue-600 text-white font-medium py-3 px-4 rounded-lg transition-colors"
            >
              Open Gradient Generator
            </button>
            
            {currentGradient && (
              <div className="mt-6 space-y-3">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Radial Gradient
                  </label>
                  <code className="block text-xs bg-gray-100 p-2 rounded break-all">
                    {currentGradient.radial}
                  </code>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Linear Gradient
                  </label>
                  <code className="block text-xs bg-gray-100 p-2 rounded break-all">
                    {currentGradient.linear}
                  </code>
                </div>
                
                <div className="grid grid-cols-3 gap-3 text-sm">
                  <div>
                    <label className="block font-medium text-gray-700">Opacity</label>
                    <span className="text-gray-600">{currentGradient.opacity.toFixed(3)}</span>
                  </div>
                  <div>
                    <label className="block font-medium text-gray-700">Texture</label>
                    <span className="text-gray-600">{currentGradient.texture.toFixed(3)}</span>
                  </div>
                  <div>
                    <label className="block font-medium text-gray-700">Grain</label>
                    <span className="text-gray-600">{currentGradient.showGrain ? 'Yes' : 'No'}</span>
                  </div>
                </div>
              </div>
            )}
          </div>
          
          {/* Preview */}
          <div className="bg-white rounded-lg shadow-sm p-6">
            <h2 className="text-xl font-semibold mb-4">Preview</h2>
            
            <div 
              className="w-full h-64 rounded-lg border-2 border-gray-200 relative overflow-hidden"
              
            >
              {currentGradient?.showGrain && (
                <div 
                  className="absolute inset-0 mix-blend-hard-light opacity-30"
                  style={{
                    backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E")`,
                    opacity: currentGradient.texture,
                  }}
                />
              )}
              
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="bg-white/90 backdrop-blur-sm px-4 py-2 rounded-lg">
                  <p className="text-gray-800 font-medium">
                    {currentGradient ? 'Custom Gradient' : 'Default Gradient'}
                  </p>
                </div>
              </div>
            </div>
            
            {/* Additional preview panels */}
            <div className="grid grid-cols-2 gap-3 mt-4">
              <div 
                className="h-16 rounded border border-gray-200"
                style={{
                  background: currentGradient?.linear || 'linear-gradient(90deg, #667eea 0%, #764ba2 100%)',
                }}
              />
              <div 
                className="h-16 rounded border border-gray-200"
                style={{
                  background: currentGradient?.radial || 'radial-gradient(circle, #667eea 0%, #764ba2 100%)',
                  filter: currentGradient?.showGrain ? 'contrast(1.1)' : 'none',
                }}
              />
            </div>
          </div>
        </div>
        
        {/* Instructions */}
        <div className="mt-8 bg-blue-50 border border-blue-200 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-blue-900 mb-2">
            How to Use
          </h3>
          <ul className="text-blue-800 space-y-1 text-sm">
            <li>• Click the button above to open the gradient generator</li>
            <li>• Left-click in the circular area to place or move the primary color dot</li>
            <li>• Right-click on dots to remove them</li>
            <li>• Use the + and - buttons to add/remove color harmony dots</li>
            <li>• Adjust opacity with the wave slider</li>
            <li>• Control texture with the circular selector</li>
            <li>• Browse color presets using the left/right arrows</li>
            <li>• Add custom colors with the color picker</li>
          </ul>
        </div>
      </div>
      
      <GradientGeneratorDialog
        open={isGradientOpen}
        onClose={() => setIsGradientOpen(false)}
        onChange={handleGradientChange}
        initialState={{
          opacity: 0.5,
          texture: 0,
          showGrain: false,
        }}
      />
    </div>
  )
}
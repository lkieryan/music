
import React, { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Card } from '~/components/ui/card'
import { Button } from '~/components/ui/button'
import { Badge } from '~/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '~/components/ui/tabs'
import { Input } from '~/components/ui/input'
import { pluginService } from '~/services/plugin-service'
import { Switch } from '~/components/ui/switch'
import { resolveImageUrl } from '~/lib/image'
import SpotifyPng from '~/assets/icons/spotify.png'
import YoutubePng from '~/assets/icons/youtube.png'
import BilibiliPng from '~/assets/icons/bilibili.png'

// Use local SVG icons instead of lucide-react
import SearchIcon from '~/assets/icons/search-glass.svg?react'
import TrashIcon from '~/assets/icons/trash.svg?react'
import SecurityIcon from '~/assets/icons/security.svg?react'
import ReloadIcon from '~/assets/icons/reload.svg?react'
import ExtensionIcon from '~/assets/icons/extension.svg?react'

// UI model for plugin info (extends backend with UI-only fields)
interface PluginInfo {
  id: string
  name: string
  display_name: string
  description: string
  version: string
  author: string
  plugin_type: 'audio-provider' | 'theme' | 'extension' | 'audio-effect' | 'lyrics-provider' | 'visualization'
  capabilities: string[]
  status: 'unloaded' | 'loading' | 'ready' | 'running' | 'stopped' | 'error'
  health: 'healthy' | 'warning' | 'unhealthy' | 'unknown'
  enabled?: boolean
  icon?: string
  installed_at: string
  permissions: string[]
  resource_usage: {
    memory_mb: number
    cpu_percent: number
    disk_mb: number
    network_requests: number
    errors: number
  }
}

export function Component() {
  const { t } = useTranslation(['app', 'common'])
  const [plugins, setPlugins] = useState<PluginInfo[]>([])
  const [searchTerm, setSearchTerm] = useState('')
  const [activeTab, setActiveTab] = useState('installed')
  const [loading, setLoading] = useState(false)
  const [selectedPlugin, setSelectedPlugin] = useState<PluginInfo | null>(null)

  // Map backend plugin to UI model with sensible defaults
  const mapBackendToUI = (p: any): PluginInfo => {
    const statusRaw = String(p.status ?? '').toLowerCase()
    const healthRaw = String(p.health ?? '').toLowerCase()
    const toStatus = (): PluginInfo['status'] => {
      switch (statusRaw) {
        case 'unloaded': return 'unloaded'
        case 'loaded':
        case 'ready': return 'ready'
        case 'running': return 'running'
        case 'stopped': return 'stopped'
        case 'error': return 'error'
        default: return 'ready'
      }
    }
    const toHealth = (): PluginInfo['health'] => {
      switch (healthRaw) {
        case 'healthy': return 'healthy'
        case 'unhealthy': return 'unhealthy'
        case 'maintenance': return 'warning'
        default: return 'unknown'
      }
    }
    const typeStr = String(p.plugin_type ?? 'extension')
    const allowed: PluginInfo['plugin_type'][] = ['audio-provider','theme','extension','audio-effect','lyrics-provider','visualization']
    const type = (allowed.includes(typeStr as any) ? typeStr : 'extension') as PluginInfo['plugin_type']
    return {
      id: p.id,
      name: p.name,
      display_name: p.display_name,
      description: p.description,
      version: p.version,
      author: p.author,
      plugin_type: type,
      capabilities: [],
      status: toStatus(),
      health: toHealth(),
      enabled: Boolean(p.enabled ?? true),
      icon: p.icon ?? undefined,
      installed_at: new Date().toISOString(),
      permissions: [],
      resource_usage: { memory_mb: 0, cpu_percent: 0, disk_mb: 0, network_requests: 0, errors: 0 },
    }
  }

  const fetchPlugins = async (opts?: { silent?: boolean }) => {
    const silent = !!opts?.silent
    try {
      if (!silent) setLoading(true)
      const list = await pluginService.getPlugins()
      setPlugins(list.map(mapBackendToUI))
    } catch (e) {
      console.error('[Extensions] Failed to fetch plugins', e)
      if (!silent) setPlugins([])
    } finally {
      if (!silent) setLoading(false)
    }
  }

  useEffect(() => {
    fetchPlugins()
  }, [])

  const filteredPlugins = plugins.filter(plugin => 
    plugin.display_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    plugin.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
    plugin.author.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const getPluginIconUrl = (p: PluginInfo): string => {
    const url = resolveImageUrl(p.icon ?? null)
    if (url) return url
    const name = (p.name || '').toLowerCase()
    if (name.includes('spotify')) return SpotifyPng
    if (name.includes('youtube')) return YoutubePng
    if (name.includes('bilibili')) return BilibiliPng
    return YoutubePng
  }

  const handleToggleEnabled = async (plugin: PluginInfo) => {
    // Optimistic update to avoid full list refresh jank
    setPlugins(prev => prev.map(p => p.id === plugin.id ? { ...p, enabled: !plugin.enabled } : p))
    try {
      if (plugin.enabled) {
        await pluginService.disablePlugin(plugin.id)
      } else {
        await pluginService.enablePlugin(plugin.id)
      }
      // Reconcile with backend in the background (silent)
      fetchPlugins({ silent: true })
    } catch (e) {
      console.error('[Extensions] Failed to toggle enabled', e)
      // Revert optimistic change on failure
      setPlugins(prev => prev.map(p => p.id === plugin.id ? { ...p, enabled: plugin.enabled } : p))
    }
  }

  // Start/Stop are unified into enable/disable persistence.

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">{t('app:pages.extensions.title')}</h1>
          <p className="text-muted-foreground mt-1">
            {t('app:pages.extensions.description')}
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="relative">
            <SearchIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
            <Input
              placeholder={t('common:actions.search')}
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-9 w-64"
            />
          </div>
          <Button variant="outline" size="sm" onClick={() => fetchPlugins()}>
            <ReloadIcon className="w-4 h-4 mr-2" />
            {t('common:actions.refresh')}
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="installed">
            {t('app:pages.extensions.tabs.installed')} ({plugins.length})
          </TabsTrigger>
          <TabsTrigger value="store">
            {t('app:pages.extensions.tabs.store')}
          </TabsTrigger>
          <TabsTrigger value="settings">
            {t('app:pages.extensions.tabs.settings')}
          </TabsTrigger>
        </TabsList>

        {/* Installed Plugins */}
        <TabsContent value="installed" className="mt-6">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <ReloadIcon className="w-6 h-6 animate-spin" />
              <span className="ml-2">{t('common:actions.loading')}</span>
            </div>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {filteredPlugins.map((plugin) => (
                <Card key={plugin.id} className="p-4 hover:shadow-md transition-shadow">
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-3">
                      <img src={getPluginIconUrl(plugin)} alt={plugin.display_name} className="w-10 h-10 rounded-md object-cover" />
                      <div>
                        <h3 className="font-semibold">{plugin.display_name}</h3>
                        <p className="text-sm text-muted-foreground">v{plugin.version}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Switch
                        checked={!!plugin.enabled}
                        onCheckedChange={() => handleToggleEnabled(plugin)}
                        aria-label={plugin.enabled ? t('common:actions.disable') : t('common:actions.enable')}
                      />
                    </div>
                  </div>
                  
                  <p className="text-sm text-muted-foreground mb-3 line-clamp-2">
                    {plugin.description}
                  </p>
                  
                  {/* Simplified card: remove capabilities and usage blocks */}
                  
                  {/* actions moved to header (top-right) */}
                </Card>
              ))}
            </div>
          )}
        </TabsContent>

        {/* Plugin Store */}
        <TabsContent value="store" className="mt-6">
          <div className="text-center py-12">
            <ExtensionIcon className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-2">
              {t('app:pages.extensions.store.coming-soon')}
            </h3>
            <p className="text-muted-foreground">
              {t('app:pages.extensions.store.description')}
            </p>
          </div>
        </TabsContent>

        {/* Settings */}
        <TabsContent value="settings" className="mt-6">
          <Card className="p-6">
            <h3 className="text-lg font-semibold mb-4">
              {t('app:pages.extensions.settings.title')}
            </h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <label className="font-medium">
                    {t('app:pages.extensions.settings.auto-update')}
                  </label>
                  <p className="text-sm text-muted-foreground">
                    {t('app:pages.extensions.settings.auto-update-desc')}
                  </p>
                </div>
                <input type="checkbox" className="toggle" defaultChecked />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <label className="font-medium">
                    {t('app:pages.extensions.settings.sandbox')}
                  </label>
                  <p className="text-sm text-muted-foreground">
                    {t('app:pages.extensions.settings.sandbox-desc')}
                  </p>
                </div>
                <input type="checkbox" className="toggle" defaultChecked />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <label className="font-medium">
                    {t('app:pages.extensions.settings.notifications')}
                  </label>
                  <p className="text-sm text-muted-foreground">
                    {t('app:pages.extensions.settings.notifications-desc')}
                  </p>
                </div>
                <input type="checkbox" className="toggle" defaultChecked />
              </div>
            </div>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Plugin Details Modal - would be implemented as a separate component */}
      {selectedPlugin && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <Card className="w-full max-w-2xl max-h-[80vh] overflow-auto">
            <div className="p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold">{selectedPlugin.display_name}</h2>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setSelectedPlugin(null)}
                >
                  ×
                </Button>
              </div>
              
              <div className="space-y-4">
                <div>
                  <h3 className="font-medium mb-2">{t('app:pages.extensions.details.permissions')}</h3>
                  <div className="flex flex-wrap gap-2">
                    {selectedPlugin.permissions.map((permission) => (
                      <Badge key={permission} variant="outline" className="flex items-center gap-1">
                        <SecurityIcon className="w-3 h-3" />
                        {permission}
                      </Badge>
                    ))}
                  </div>
                </div>
                
                <div>
                  <h3 className="font-medium mb-2">{t('app:pages.extensions.details.resource-usage')}</h3>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-muted-foreground">{t('app:pages.extensions.memory')}:</span>
                      <span className="ml-2 font-mono">{selectedPlugin.resource_usage.memory_mb.toFixed(1)} MB</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">{t('app:pages.extensions.cpu')}:</span>
                      <span className="ml-2 font-mono">{selectedPlugin.resource_usage.cpu_percent.toFixed(1)}%</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">{t('app:pages.extensions.disk')}:</span>
                      <span className="ml-2 font-mono">{selectedPlugin.resource_usage.disk_mb.toFixed(1)} MB</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">{t('app:pages.extensions.requests')}:</span>
                      <span className="ml-2 font-mono">{selectedPlugin.resource_usage.network_requests}</span>
                    </div>
                  </div>
                </div>
                
                <div className="flex gap-2 pt-4">
                  {/* TODO: uninstall flow to be implemented */}
                  <Button
                    variant="primary"
                    onClick={() => setSelectedPlugin(null)}
                    buttonClassName="bg-red-600 hover:bg-red-700 text-white"
                  >
                    <TrashIcon className="w-4 h-4 mr-2" />
                    {t('common:actions.uninstall')}
                  </Button>
                </div>
              </div>
            </div>
          </Card>
        </div>
      )}
    </div>
  )
}

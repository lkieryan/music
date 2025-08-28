import { invoke } from '@tauri-apps/api/core';

// Plugin information structure
export interface PluginInfo {
  /// Plugin ID
  id: string;
  
  /// Plugin name
  name: string;
  
  /// Display name
  display_name: string;
  
  /// Description
  description: string;
  
  /// Version
  version: string;
  
  /// Author
  author: string;
  
  /// Plugin type
  plugin_type: string;
  
  /// Current status
  status: string;
  
  /// Health status
  health: string;
  
  /// Whether the plugin is enabled
  enabled: boolean;
  
  /// Icon path (optional; file path or relative path)
  icon?: string;
}

class PluginService {
  // Get all plugins
  async getPlugins(): Promise<PluginInfo[]> {
    try {
      const plugins = await invoke<PluginInfo[]>('get_plugins');
      return plugins;
    } catch (error) {
      console.error('[PluginService] 获取插件列表失败:', error);
      throw error;
    }
  }

  // Get plugin by ID
  async getPlugin(pluginId: string): Promise<PluginInfo> {
    try {
      const plugin = await invoke<PluginInfo>('get_plugin', { plugin_id: pluginId, pluginId });
      return plugin;
    } catch (error) {
      console.error('[PluginService] 获取插件信息失败:', error);
      throw error;
    }
  }

  // Enable a plugin
  async enablePlugin(pluginId: string): Promise<void> {
    try {
      await invoke('enable_plugin', { plugin_id: pluginId, pluginId });
    } catch (error) {
      console.error('[PluginService] 启用插件失败:', error);
      throw error;
    }
  }

  // Disable a plugin
  async disablePlugin(pluginId: string): Promise<void> {
    try {
      await invoke('disable_plugin', { plugin_id: pluginId, pluginId });
    } catch (error) {
      console.error('[PluginService] 禁用插件失败:', error);
      throw error;
    }
  }

  // Start a plugin
  async startPlugin(pluginId: string): Promise<void> {
    try {
      await invoke('start_plugin', { plugin_id: pluginId, pluginId });
    } catch (error) {
      console.error('[PluginService] 启动插件失败:', error);
      throw error;
    }
  }

  // Stop a plugin
  async stopPlugin(pluginId: string): Promise<void> {
    try {
      await invoke('stop_plugin', { plugin_id: pluginId, pluginId });
    } catch (error) {
      console.error('[PluginService] 停止插件失败:', error);
      throw error;
    }
  }

  // Load a plugin from file
  async loadPlugin(pluginPath: string): Promise<void> {
    try {
      await invoke('load_plugin', { plugin_path: pluginPath, pluginPath });
    } catch (error) {
      console.error('[PluginService] 加载插件失败:', error);
      throw error;
    }
  }
}

// Export singleton instance
export const pluginService = new PluginService();

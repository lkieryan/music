// Minimal safe wrapper around @tauri-apps/api/invoke
// If running in web (non-tauri), we provide a fallback no-op implementation for dev.

export async function invoke<T = any>(cmd: string, args?: Record<string, any>): Promise<T> {
  // @ts-ignore
  const w: any = typeof window !== 'undefined' ? window : {}
  const api = w.__TAURI__?.core || w.__TAURI__?.tauri
  if (!api?.invoke) {
    // Non-tauri environment: allow dev with mock
    console.warn('[tauri] invoke fallback for', cmd, args)
    return Promise.reject(new Error('Not running in Tauri environment'))
  }
  return api.invoke<T>(cmd, args)
}

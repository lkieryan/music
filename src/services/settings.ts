import { invoke } from '@tauri-apps/api/core'
import { listen as tauriListen } from '@tauri-apps/api/event'
import { debounce } from 'es-toolkit/compat'

export async function loadSettings<T = any>(key: string): Promise<T | undefined> {
  return invoke('load_selective', { key }).catch(() => undefined)
}

export async function saveSettings<T = any>(key: string, value: T): Promise<void> {
  await invoke('save_selective', { key, value })
}

export async function getSecure<T = any>(key: string): Promise<T | undefined> {
  return invoke('get_secure', { key }).catch(() => undefined)
}

export async function setSecure<T = any>(key: string, value: T | null): Promise<void> {
  await invoke('set_secure', { key, value })
}

export async function loadDomain<T = any>(domain: string = ''): Promise<T> {
  return invoke('load_domain', { domain })
}
  
export async function saveDomainPartial(domain: string = '', patch: Record<string, any>): Promise<void> {
  await invoke('save_domain_partial', { domain, patch })
}

// Create a setter that updates local state immediately and saves to backend (debounced)
export function createBackendBoundSetter<T extends object>(
  setLocal: (key: keyof T, value: any) => void,
  mapKey: (key: keyof T) => string | null,
  options?: { debounceMs?: number }
) {
  const delay = options?.debounceMs ?? 200
  const debounced = debounce((k: string, v: any) => {
    saveSettings(k, v).catch(() => {})
  }, delay)

  return function setBoth<K extends keyof T>(key: K, value: T[K]) {
    setLocal(key, value)
    const backendKey = mapKey(key)
    if (backendKey) debounced(backendKey, value)
  }
}

export type DomainBinding<T extends object> = {
  hydrate: () => Promise<void>
  listen: (onAfterChange?: (k: keyof T, v: any) => void) => () => void
  set: <K extends keyof T>(key: K, value: T[K]) => void
}

// Create a domain binding that:
// - hydrates from backend with a single domain fetch (plus root)
// - listens for settings-changed to keep atoms in sync
// - returns a setter that updates local immediately and persists to the chosen path
export function createDomainBinding<UI extends Record<string, any>, BE extends Record<string, any> = any>(
  params: {
    domain: string
    // the default factory is used to know the allowed keys
    defaultFactory: () => UI
    setLocal: (key: keyof UI, value: any) => void
    // optional backend key override: frontKey -> backend key (snake_case or otherwise)
    keyMap?: Partial<Record<keyof UI, string>>
    // optional: where to save if not discovered during hydrate
    pathPolicy?: Partial<Record<keyof UI, 'root' | 'domain'>>
    // optional: override debounce
    debounceMs?: number
  }
): DomainBinding<UI> {
  const { domain, defaultFactory, setLocal, debounceMs = 200, keyMap = {} as any, pathPolicy = {} as any } = params
  const allowed = new Set(Object.keys(defaultFactory()))
  // record where each key should be saved: 'domain' or 'root'
  const savePath: Record<string, 'domain' | 'root'> = {}

  const snakeOf = (frontKey: string) => (keyMap as any)[frontKey] ?? camelToSnake(frontKey)

  // reverse map from backend snake key -> front key
  const frontOfSnake = (snake: string) => {
    for (const [fk, sk] of Object.entries(keyMap as Record<string, string>)) {
      if (sk === snake) return fk
    }
    return snakeToCamel(snake)
  }

  async function hydrate() {
    const [domainObj, rootObj] = await Promise.all([
      domain ? loadDomain<any>(domain).catch(() => ({})) : Promise.resolve({}),
      loadDomain<any>('').catch(() => ({})),
    ])
    const keys = Array.from(allowed)
    for (const k of keys) {
      const snake = snakeOf(k)
      let value: any | undefined = undefined
      if (domain && domainObj && typeof domainObj === 'object') {
        value = (domainObj as any)[snake]
        if (value !== undefined) savePath[k] = 'domain'
      }
      if (value === undefined && rootObj && typeof rootObj === 'object') {
        value = (rootObj as any)[snake]
        if (value !== undefined) savePath[k] = 'root'
      }
      if (value !== undefined) setLocal(k as keyof UI, value)
    }
  }

  function listen(onAfterChange?: (k: keyof UI, v: any) => void) {
    let unsub: any
    tauriListen('settings-changed', (e: any) => {
      console.log("settings-changed",e)
      const [fullKey, value] = Array.isArray(e?.payload) ? e.payload : []
      if (!fullKey) return
      const path = String(fullKey).replace(/^prefs\./, '')
      // match domain.key or root key
      if (domain && path.startsWith(`${domain}.`)) {
        const seg = path.slice(domain.length + 1)
        const snake = seg.split('.').pop() || seg
        const fk = frontOfSnake(snake)
        if (allowed.has(fk)) {
          setLocal(fk as keyof UI, value)
          onAfterChange?.(fk as keyof UI, value)
        }
        return
      }
      // root-level (no dot)
      if (!path.includes('.')) {
        const snake = path
        const fk = frontOfSnake(snake)
        if (allowed.has(fk)) {
          setLocal(fk as keyof UI, value)
          onAfterChange?.(fk as keyof UI, value)
        }
      }
    }).then((fn: any) => (unsub = fn))
    return () => { if (unsub) unsub() }
  }

  const set = createBackendBoundSetter<UI>(
    (key: keyof UI, value: any) => setLocal(key as any, value),
    (key: keyof UI) => {
      const k = String(key)
      const where = savePath[k] ?? (domain ? 'domain' : 'root')
      const snake = snakeOf(k)
      if (where === 'domain' && domain) return `${domain}.${snake}`
      return snake
    },
    { debounceMs }
  )

  return { hydrate, listen, set }
}

function snakeToCamel(str: string): string {
  return str.replace(/_([a-z])/g, (_, c) => c.toUpperCase())
}
  
function camelToSnake(str: string): string {
  return str
    .replace(/([A-Z])/g, '_$1')
    .replace(/__/g, '_')
    .toLowerCase()
}

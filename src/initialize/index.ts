import { initI18n } from "~/i18n"
import { appLog } from "~/lib/log"

export const initializeApp = async () => {
  await apm("i18n", initI18n)

  const unlistenSettings = await apm("settings-hydrate", async () => {
    const { settingsHydrate } = await import('~/initialize/settings-hydrate')
    return settingsHydrate()
  })

  // 初始化扫描器服务
  await apm("scanner-service", async () => {
    const { scannerService } = await import('~/services/scanner-service')
    // 扫描器服务在导入时自动初始化
    return Promise.resolve()
  })

  // keep reference if you need to unlisten on teardown
  void unlistenSettings
}

const apm = async (label: string, fn: () => Promise<any> | any) => {
  const start = Date.now()
  const result = await fn()
  const end = Date.now()
  appLog(`${label} took ${end - start}ms`)
  return result
}

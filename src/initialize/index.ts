import { initI18n } from "~/i18n"
import { appLog } from "~/lib/log"


export const initializeApp = async () => {
  await apm("i18n", initI18n)
}

const apm = async (label: string, fn: () => Promise<any> | any) => {
  const start = Date.now()
  const result = await fn()
  const end = Date.now()
  appLog(`${label} took ${end - start}ms`)
  return result
}
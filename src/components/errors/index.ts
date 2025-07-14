import { lazy } from "react"

import { ErrorComponentType } from "./enum"

const ErrorFallbackMap = {
  [ErrorComponentType.Modal]: lazy(() => import("./modal-error")),
  [ErrorComponentType.FeedNotFound]: lazy(() => import("./feed-not-found")),
}

export const getErrorFallback = (type: ErrorComponentType) => ErrorFallbackMap[type]

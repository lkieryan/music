// import { nextFrame } from "~/lib/dom"

// import { createErrorToaster } from "~/lib/error-parser"

export class CustomSafeError extends Error {
  constructor(message: string, toast?: boolean) {
    super(message)
    // if (toast) {
    //   nextFrame(() => createErrorToaster(message)(this))
    // }
  }
}

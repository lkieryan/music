import { useAtomValue } from "jotai"
import { useEffect } from "react"

import { modalStackAtom } from "./atom"

export const useModalStackCalculationAndEffect = () => {
  const stack = useAtomValue(modalStackAtom)
  const topModalIndex = stack.findLastIndex((item) => item.modal)
  const overlayIndex = stack.findLastIndex((item) => item.overlay || item.modal)
  const overlayOptions = stack[overlayIndex]?.overlayOptions

  const hasModalStack = stack.length > 0
  const topModalIsNotSetAsAModal = topModalIndex !== stack.length - 1

  useEffect(() => {
    // Keep pointer-events enabled globally and rely on overlay/container z-index to block background.
    // Toggling pointer-events on the root disables descendants as well, so don't do it here.
    document.documentElement.style.pointerEvents = "auto"
    document.documentElement.dataset.hasModal = hasModalStack.toString()
  }, [hasModalStack])

  return {
    overlayOptions,
    topModalIndex,
    hasModalStack,
    topModalIsNotSetAsAModal,
  }
}

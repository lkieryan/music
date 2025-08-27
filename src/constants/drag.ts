/**
 * Default selectors for elements that should not trigger window dragging
 */
export const DEFAULT_NO_DRAG_SELECTORS = [
  '.no-drag',
  'button',
  'input',
  'select',
  'textarea',
  '[role="button"]',
  '[tabindex]',
  '[contenteditable]',
  '.draggable', // For custom draggable elements
  '[draggable="true"]'
] as const

/**
 * Join selectors into a CSS selector string
 */
export const joinSelectors = (selectors: readonly string[]): string => {
  return selectors.join(', ')
}

/**
 * Default no-drag selector string
 */
export const DEFAULT_NO_DRAG_SELECTOR = joinSelectors(DEFAULT_NO_DRAG_SELECTORS)

/**
 * Create a custom no-drag selector by combining default selectors with additional ones
 * @param additionalSelectors - Additional selectors to exclude from dragging
 * @returns Combined selector string
 */
export const createNoDragSelector = (additionalSelectors: string[] = []): string => {
  return joinSelectors([...DEFAULT_NO_DRAG_SELECTORS, ...additionalSelectors])
}

/**
 * Common interactive element selectors that should not trigger dragging
 */
export const INTERACTIVE_ELEMENT_SELECTORS = [
  'a[href]',
  '[onclick]',
  '.clickable',
  '.interactive',
  '.menu-item',
  '.dropdown-item'
] as const

/**
 * Create a comprehensive no-drag selector including interactive elements
 * @param additionalSelectors - Additional selectors to exclude from dragging
 * @returns Comprehensive selector string
 */
export const createComprehensiveNoDragSelector = (additionalSelectors: string[] = []): string => {
  return joinSelectors([
    ...DEFAULT_NO_DRAG_SELECTORS,
    ...INTERACTIVE_ELEMENT_SELECTORS,
    ...additionalSelectors
  ])
}
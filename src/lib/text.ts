// Lightweight HTML -> plain text utility for UI safety
// - Uses DOM parsing when available to properly decode entities
// - Falls back to regex stripping in non-DOM environments
export function stripHtml(input?: string | null): string {
  if (input == null) return ''
  const s = String(input)
  if (!/[<>]/.test(s)) return s
  try {
    if (typeof window !== 'undefined' && typeof document !== 'undefined') {
      const div = document.createElement('div')
      // Preserve rough spacing for <br> by converting to spaces
      div.innerHTML = s.replace(/<br\s*\/?>(\n)?/gi, ' ')
      const text = div.textContent || div.innerText || ''
      return text.trim()
    }
  } catch {
    // ignore
  }
  // Fallback: naive tag removal + whitespace collapse
  return s.replace(/<[^>]*>/g, '').replace(/\s+/g, ' ').trim()
}


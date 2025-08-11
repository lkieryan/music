export async function settingsHydrate() {
  // wire setting change listener and perform initial hydration
  try {
    // Lazy import to avoid circular deps
    const { getGeneralBinding } = await import('~/atoms/settings/general')
    const generalBinding = getGeneralBinding()

    // hydrate from backend first
    await generalBinding.hydrate()

    // Listen to backend changes and reflect in UI atoms
    const unlisten = generalBinding.listen(async (k, value) => {
      console.log('settings-changed', k, value)
    })

    // Return unlisten for optional teardown by caller
    return unlisten
  } catch (e) {
    console.warn('[prefs] init sync failed', e)
    return () => {}
  }
}

export async function settingsHydrate() {
  // wire setting change listener and perform initial hydration
  try {
    // Lazy import to avoid circular deps
    const { getGeneralBinding } = await import('~/atoms/settings/general')
    const { getMusicBinding } = await import('~/atoms/settings/music')
    const generalBinding = getGeneralBinding()
    const musicBinding = getMusicBinding()

    // hydrate from backend first
    await generalBinding.hydrate()
    await musicBinding.hydrate()

    // Listen to backend changes and reflect in UI atoms
    const unlistenGeneral = generalBinding.listen(async (k, value) => {
      console.log('settings-changed', k, value)
    })
    const unlistenMusic = musicBinding.listen(async (k, value) => {
      console.log('music-settings-changed', k, value)
    })

    // Return unlisten for optional teardown by caller
    return () => { unlistenGeneral(); unlistenMusic() }
  } catch (e) {
    console.warn('[prefs] init sync failed', e)
    return () => {}
  }
}

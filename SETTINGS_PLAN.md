# Settings Integration Plan

Goal: Make settings centrally defined and persisted in the backend (Tauri/Rust), with the frontend acting as the presentation layer. Implement a clean bridge (hydrate/save/event) so UI stays consistent with backend and sensitive data uses secure storage.

Phases

1) Backend as Source of Truth (already available)
- SettingsConfig persists to config.json, exposes commands: load_selective, save_selective, load_selective_array, get_secure, set_secure.
- initial() writes defaults for missing keys, handle_settings_changes broadcasts UI-safe changes and triggers side-effects (scanner, providers, autostart, timers).

2) Frontend storage & namespace hygiene
- Add a storage namespace helper (src/lib/ns.ts) and use it in createSettingAtom to avoid key collisions.
- Keep UI-only settings in local storage via Jotai; keep backend-driven settings in memory only or cache with force-hydrate on startup.

3) Frontend bridge modules
- preference-bridge.ts: createBackendBoundSetter() to combine local state update and debounced backend save. Prevent chattiness for text inputs.
- preference-sync.ts: hydrateFromBackend() to load backend values on app init; listenPreferenceChanged() to subscribe to "settings-changed" and update local atoms to keep multiple windows in sync.
- services/settings.ts: add getSecure/setSecure helpers to call Tauri secure APIs.

4) App initialization
- Wire hydrateFromBackend + listenPreferenceChanged in src/initialize/index.ts so pages render with backend values immediately and stay in sync.

5) Mapping & domains
- Introduce a central mapping of frontend atoms/keys to backend dot-paths (e.g., scanInterval -> scan_interval, music/exclude paths, language). Start minimal and expand as pages are implemented.
- For checkbox-array settings (e.g., language), optionally add an active single-value on the backend or provide a translator in the bridge.

6) Sensitive fields
- Use setSecure/getSecure for tokens/passwords. Do not store plaintext in localStorage. UI shows configured/clear/re-set without echoing secrets.

7) Quality & resilience
- Debounce saves for text inputs (150-300ms). Toggle saves immediate.
- Avoid event feedback loops via source tokens or equality checks.
- Consider batch domain load (future): load_domain('general') returning merged defaults to reduce multiple invokes.

Deliverables in this iteration
- src/lib/ns.ts namespacing helper
- Patch src/atoms/helper/setting.ts to use namespaced key
- src/services/settings.ts: add getSecure/setSecure
- src/services/settings.ts: createBackendBoundSetter()
- src/services/settings.ts: hydrate + event listener skeleton + example mapping
- Wire into src/initialize/index.ts

Next steps (future)
- Expand mapping to actual implemented settings pages.
- Optionally add batch load Tauri command (load_domain) on the backend.
- Migrate sensitive integration settings to set_secure/get_secure UI flows.

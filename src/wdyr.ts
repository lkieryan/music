if (import.meta.env.DEV) {
  // avoid top-level await for broader browser compatibility
  void import("react-scan").then(({ scan }) => {
    scan({ enabled: false, log: false, showToolbar: true })
  })
}

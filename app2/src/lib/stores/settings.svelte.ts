class SettingsStore {
  pageLimit: number = $state(12)
  showQuoteTokens: boolean = $state(false)
}

export const settingsStore = new SettingsStore()

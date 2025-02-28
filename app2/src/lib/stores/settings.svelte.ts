class SettingsStore {
  pageLimit: number = $state(10)
  showQuoteTokens: boolean = $state(true)
}

export const settingsStore = new SettingsStore()

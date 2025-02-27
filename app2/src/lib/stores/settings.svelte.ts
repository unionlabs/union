class SettingsStore {
  pageLimit: number = $state(10)
}

export const settingsStore = new SettingsStore()

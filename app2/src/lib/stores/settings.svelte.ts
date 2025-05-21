// Careful, this is untyped. Ensure the overrides actually exist in SettingsStore
const editionDefaults: Record<string, Record<string, any>> = {
  btc: {
    mainnetOnly: true,
  },
  app: {
    mainnetOnly: false,
  },
}

class SettingsStore {
  pageLimit: number = $state(12)
  showQuoteTokens: boolean = $state(false)
  showDeveloperChainDetails: boolean = $state(false)
  mainnetOnly: boolean = $state(false)

  setEditionDefaults(edition: string) {
    if (editionDefaults[edition]) {
      const defaults = editionDefaults[edition]
      Object.keys(defaults).forEach(key => {
        if (key in this) {
          // @ts-ignore: Dynamic property assignment
          this[key] = defaults[key]
        }
      })
    }
  }
}

export const settingsStore = new SettingsStore()

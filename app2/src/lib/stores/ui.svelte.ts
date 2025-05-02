import base64Icon from "$lib/config/base64.txt?raw"
import { themes } from "$lib/themes"
import type { Edition } from "$lib/themes"

const projectIds: Record<Edition, string> = {
  app: "f544d5ee6eb61962408fd456c114e9ed",
  btc: "49fe74ca5ded7142adefc69a7788d14a",
}

class UiStore {
  walletModalOpen: boolean = $state(false)
  settingsModalOpen: boolean = $state(false)
  errorsModalOpen: boolean = $state(false)
  showZeroBalances: boolean = $state(false)
  showDeveloperPages: boolean = $state(false)
  filterWhitelist: boolean = $state(true)

  edition: Edition = $state("app")
  overrideEdition: Edition | null = $state(null)

  activeEdition: Edition = $derived(this.overrideEdition ?? this.edition)
  theme = $derived(themes[this.activeEdition])

  appInfo = $derived({
    base64Icon,
    name: "Union",
    baseUrl: `https://${this.activeEdition}.union.build`,
    docs: "https://docs.union.build",
    iconUrl: "https://app.union.build/images/logo.png",
    projectId: projectIds[this.activeEdition],
  })

  private closeAllModals() {
    this.walletModalOpen = false
    this.settingsModalOpen = false
    this.errorsModalOpen = false
  }

  openWalletModal() {
    this.closeAllModals()
    this.walletModalOpen = true
  }

  closeWalletModal() {
    this.walletModalOpen = false
  }

  openSettingsModal() {
    this.closeAllModals()
    this.settingsModalOpen = true
  }

  closeSettingsModal() {
    this.settingsModalOpen = false
  }

  openErrorsModal() {
    this.errorsModalOpen = true
  }

  closeErrorsModal() {
    this.closeAllModals()
    this.errorsModalOpen = false
  }
}

export const uiStore = new UiStore()

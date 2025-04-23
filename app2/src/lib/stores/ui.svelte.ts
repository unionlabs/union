import { themes } from "$lib/themes"
import type { Edition } from "$lib/themes"

class UiStore {
  // TODO: make Option<"wallet"|"settings"|"errors">
  walletModalOpen: boolean = $state(false)
  settingsModalOpen: boolean = $state(false)
  errorsModalOpen: boolean = $state(false)
  showZeroBalances: boolean = $state(false)
  showDeveloperPages: boolean = $state(false)
  edition: Edition = $state("app")
  overrideEdition: Edition | null = $state(null)

  get activeEdition() {
    return this.overrideEdition ?? this.edition
  }

  get theme() {
    return themes[this.activeEdition]
  }

  get accentColor() {
    return this.theme.accent
  }

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

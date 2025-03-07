class UiStore {
  // TODO: make Option<"wallet"|"settings"|"errors">
  walletModalOpen: boolean = $state(false)
  settingsModalOpen: boolean = $state(false)
  errorsModalOpen: boolean = $state(true)

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

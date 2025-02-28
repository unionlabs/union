class UiStore {
  walletModalOpen: boolean = $state(false)
  settingsModalOpen: boolean = $state(false)

  private closeAllModals() {
    this.walletModalOpen = false
    this.settingsModalOpen = false
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
}

export const uiStore = new UiStore()

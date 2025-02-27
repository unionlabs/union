class UiStore {
  walletModalOpen: boolean = $state(false)
  settingsModalOpen: boolean = $state(false)

  openWalletModal() {
    this.walletModalOpen = true
  }

  closeWalletModal() {
    this.walletModalOpen = false
  }

  openSettingsModal() {
    this.settingsModalOpen = true
  }

  closeSettingsModal() {
    this.settingsModalOpen = false
  }
}

export const uiStore = new UiStore()

class UiStore {
  walletModalOpen: boolean = $state(false)

  openWalletModal() {
    this.walletModalOpen = true
  }

  closeWalletModal() {
    this.walletModalOpen = false
  }
}

export const uiStore = new UiStore()

import { Option } from "effect"
import type { LockedTransfer } from "./locked-transfer.ts"

// Create a store for the locked transfer
class LockedTransferStore {
  transfer = $state<Option.Option<LockedTransfer>>(Option.none())

  lock(lockedTransfer: LockedTransfer) {
    this.transfer = Option.some(lockedTransfer)
  }

  unlock() {
    this.transfer = Option.none()
  }

  get() {
    return this.transfer
  }
}

export const lockedTransferStore = new LockedTransferStore()

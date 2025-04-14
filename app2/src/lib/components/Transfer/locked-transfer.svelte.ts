import {Option} from "effect"
import type {LockedTransfer} from "./locked-transfer.ts"

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

  isLocked() {
    return Option.isSome(this.transfer)
  }

  reset() {
    this.unlock()
    this.transfer = Option.none()
  }
}

export const lockedTransferStore = new LockedTransferStore()

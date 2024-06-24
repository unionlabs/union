import { timestamp } from "../utilities/index.ts"
import { cosmwasmTransferSimulate, cosmwasmTransfer } from "./transfer.ts"

export const cosmwasmActions = () => ({
  transfer: cosmwasmTransfer,
  simulate: cosmwasmTransferSimulate
})

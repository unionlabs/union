import { timestamp } from "../utilities/index.ts"
import { ibcTransfer, ibcTransferSimulate } from "./transfer.ts"

export const ibcActions = () => ({
  transfer: ibcTransfer,
  simulate: ibcTransferSimulate
})

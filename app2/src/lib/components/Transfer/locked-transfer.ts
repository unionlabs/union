import {Option} from "effect"
import type {Chain, Channel, Token} from "@unionlabs/sdk/schema"
import type {TransferStep} from "./transfer-step.ts"

/**
 * LockedTransfer stores a snapshot of critical transfer values
 * to ensure they don't change after the user proceeds past the filling step
 */
export class LockedTransfer {
  sourceChain: Chain
  destinationChain: Chain
  channel: Channel
  parsedAmount: string
  baseToken: Token
  steps: Array<TransferStep>

  constructor(
    sourceChain: Chain,
    destinationChain: Chain,
    channel: Channel,
    parsedAmount: string,
    baseToken: Token,
    steps: Array<TransferStep>
  ) {
    this.sourceChain = sourceChain
    this.destinationChain = destinationChain
    this.channel = channel
    this.parsedAmount = parsedAmount
    this.baseToken = baseToken
    this.steps = steps
  }

  /**
   * Creates a LockedTransfer from the current state of a transfer
   * Returns None if any required values are missing
   */
  static fromTransfer(
    sourceChain: Option.Option<Chain>,
    destinationChain: Option.Option<Chain>,
    channel: Option.Option<Channel>,
    parsedAmount: Option.Option<string>,
    baseToken: Option.Option<Token>,
    steps: Option.Option<Array<TransferStep>>
  ): Option.Option<LockedTransfer> {
    return Option.all([
      sourceChain,
      destinationChain,
      channel,
      parsedAmount,
      baseToken,
      steps
    ]).pipe(Option.map(([sc, dc, ch, pa, bt, st]) => new LockedTransfer(sc, dc, ch, pa, bt, st)))
  }

  /**
   * Gets the current step based on page index
   */
  getStep(pageIndex: number): Option.Option<TransferStep> {
    return pageIndex >= 0 && pageIndex < this.steps.length
      ? Option.some(this.steps[pageIndex])
      : Option.none()
  }
}

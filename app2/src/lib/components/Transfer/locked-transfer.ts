import { Option } from "effect"
import type { Chain, Channel } from "@unionlabs/sdk/schema"
import type { TransferStep } from "./transfer-step.ts"

/**
 * LockedTransfer stores a snapshot of critical transfer values
 * to ensure they don't change after the user proceeds past the filling step
 */
export class LockedTransfer {
  sourceChain: Chain
  destinationChain: Chain
  channel: Channel
  steps: Array<TransferStep>

  constructor(
    sourceChain: Chain,
    destinationChain: Chain,
    channel: Channel,
    steps: Array<TransferStep>
  ) {
    this.sourceChain = sourceChain
    this.destinationChain = destinationChain
    this.channel = channel
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
    parsedAmountL: Option.Option<string>,
    steps: Option.Option<Array<TransferStep>>
  ): Option.Option<LockedTransfer> {
    return Option.all([sourceChain, destinationChain, channel, steps]).pipe(
      Option.map(([sc, dc, ch, st]) => new LockedTransfer(sc, dc, ch, st))
    )
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

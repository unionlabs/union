<script lang="ts">
import { page } from "$app/state"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import PacketComponent from "$lib/components/model/PacketComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { packetDetailsQuery } from "$lib/queries/packet-details.svelte"
import { transferByPacketHashQuery } from "$lib/queries/transfer-by-hash.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { packetDetails } from "$lib/stores/packets.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import { cosmosStore } from "$lib/wallet/cosmos"
import { getChain, TokenRawDenom, TransactionHash } from "@unionlabs/sdk/schema"
import { Data, Fiber, Option } from "effect"
import { onMount } from "svelte"

// Store for the transfer details
import LoadingSpinnerIcon from "$lib/components/icons/LoadingSpinnerIcon.svelte"
import RotateLeftIcon from "$lib/components/icons/RotateLeftIcon.svelte"
import SharpCheckIcon from "$lib/components/icons/SharpCheckIcon.svelte"
import SharpDoubleCheckIcon from "$lib/components/icons/SharpDoubleCheckIcon.svelte"
import SharpWarningIcon from "$lib/components/icons/SharpWarningIcon.svelte"
import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte"
import PacketTracesComponent from "$lib/components/model/PacketTracesComponent.svelte"
import A from "$lib/components/ui/A.svelte"
import Button from "$lib/components/ui/Button.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import { finalityDelays, settlementDelays } from "$lib/constants/settlement-times"
import { runFork, runPromise } from "$lib/runtime"
import { transferDetails } from "$lib/stores/transfer-details.svelte"
import { fromHex } from "viem"
import Layout from "../../../+layout.svelte"
import type { PageData } from "./$types"

type Props = {
  data: PageData
}

const { data }: Props = $props()

// State for packet details visibility
let showPacketDetails = $state(false)

let fiber: Fiber.Fiber<any, any>

onMount(() => {
  fiber = runFork(transferByPacketHashQuery(data.packetHash))
  packetDetails.runEffect(packetDetailsQuery(data.packetHash))

  return async () => {
    await runPromise(Fiber.interrupt(fiber))
    transferDetails.data = Option.none()
    transferDetails.error = Option.none()

    // Clean up packet details
    packetDetails.interruptFiber()
  }
})

type SimpleTransferStatus = Data.TaggedEnum<{
  NoDetails: {}
  TimeoutPending: {}
  TimeoutSubmitted: {
    tx_hash: TransactionHash
  }
  SuccessAck: {}
  Success: {}
  FailedAck: {}
  Failed: {}
  InProgress: {}
}>
export const SimpleTransferStatus = Data.taggedEnum<SimpleTransferStatus>()

const simpleStatus = $derived.by(() => {
  if (Option.isNone(transferDetails.data) || Option.isNone(packetDetails.data)) {
    return SimpleTransferStatus.NoDetails()
  }
  const transfer = transferDetails.data.value
  const packet = packetDetails.data.value

  if (Option.isSome(transfer.transfer_timeout_transaction_hash)) {
    return SimpleTransferStatus.TimeoutSubmitted({
      tx_hash: transfer.transfer_timeout_transaction_hash.value,
    })
  }

  const HAS_WRITE_ACK = transfer.traces.some(t =>
    t.type === "WRITE_ACK" && Option.isSome(t.transaction_hash)
  )
  const HAS_PACKET_ACK = transfer.traces.some(t =>
    t.type === "PACKET_ACK" && Option.isSome(t.transaction_hash)
  )

  // timeout_timestamp can be 0, in which there is no timeout
  if (packet.timeout_timestamp && !HAS_WRITE_ACK) {
    // Convert current time to nanoseconds since UNIX epoch
    const currentNanos = BigInt(Date.now()) * 1000000n
    // NOTE: this is assuming that the indexer is up-to-date
    // If indexing is unhealthy for destination chain, it will incorrectly show that a
    // timeout is pending
    if (currentNanos >= packet.timeout_timestamp) {
      return SimpleTransferStatus.TimeoutPending()
    }
  }

  if (HAS_PACKET_ACK) {
    return transfer.success.pipe(Option.match({
      onNone: () => SimpleTransferStatus.Failed(),
      onSome: (s) =>
        s === true ? SimpleTransferStatus.SuccessAck() : SimpleTransferStatus.FailedAck(),
    }))
  }

  if (HAS_WRITE_ACK) {
    return transfer.success.pipe(Option.match({
      onNone: () => SimpleTransferStatus.InProgress(),
      onSome: (s) => s === true ? SimpleTransferStatus.Success() : SimpleTransferStatus.Failed(),
    }))
  }
  return SimpleTransferStatus.InProgress()
})

const suggestTokenToWallet = async (chain_id: string, denom: TokenRawDenom) => {
  console.log("suggest token", chain_id, denom)
  const denomCosmwasm = fromHex(denom, "string")

  if (window.keplr) {
    console.log("adding to keplr")
    const x = await window.keplr.suggestToken(chain_id, denomCosmwasm)
    console.log(x)
  }

  if (window.leap) {
    console.log("adding to leap")
    const x = await window.leap.suggestCW20Token(chain_id, denomCosmwasm)
    console.log(x)
  }
}
</script>

<Sections>
  <Card
    class="overflow-auto"
    divided
  >
    <div class="p-4">Transfer Details</div>
    <div>
      {#if Option.isSome(transferDetails.error)}
        <ErrorComponent error={transferDetails.error.value} />
      {:else if Option.isSome(transferDetails.data) && Option.isSome(chains.data)}
        {@const transfer = transferDetails.data.value}
        {@const chainsList = chains.data.value}
        {@const sourceChain = getChain(
          chainsList,
          transfer.source_chain.universal_chain_id,
        )}
        {@const destChain = getChain(
          chainsList,
          transfer.destination_chain.universal_chain_id,
        )}

        <div class="space-y-8">
          <!-- Chain and Token Transfer Display -->
          <div class="flex flex-col gap-6">
            <div class="flex flex-1 items-center justify-between pt-6 px-4">
              <div class="text-2xl">
                {#if Option.isSome(destChain)}
                  <TokenComponent
                    chain={destChain.value}
                    denom={transfer.quote_token}
                    amount={transfer.quote_amount}
                  />
                {/if}
              </div>
              <div class="flex items-center gap-4">
                {#if simpleStatus._tag === "InProgress"}
                  <SpinnerIcon class="size-6" />
                  <p>In progress</p>
                {:else if simpleStatus._tag === "SuccessAck"}
                  <!--
                    Different icon for success + ack, but same text.
                    It is a useful detail for developers, but end-users should 
                    not care about the difference between success and success + ack.
                  !-->
                  <SharpDoubleCheckIcon class="size-6 text-accent" />
                  <p class="text-babylon">Received</p>
                {:else if simpleStatus._tag === "Success"}
                  <SharpCheckIcon class="size-6 text-accent" />
                  <p class="text-babylon">Received</p>
                {:else if simpleStatus._tag === "Failed"}
                  <SharpWarningIcon class="size-6 text-yellow-500 self-center" />
                  <div class="flex flex-col">
                    <p class="text-babylon">Failed transfer</p>
                    <p class="text-babylon text-xs text-zinc-400">
                      Will be refunded
                    </p>
                  </div>
                {:else if simpleStatus._tag === "FailedAck"}
                  <SharpWarningIcon class="size-6 text-yellow-500 self-center" />
                  <div class="flex flex-col">
                    <p class="text-babylon">Failed transfer</p>
                    <p class="text-babylon text-xs text-zinc-400">
                      Has been refunded
                    </p>
                  </div>
                {:else if simpleStatus._tag === "TimeoutPending"}
                  <RotateLeftIcon class="size-6 text-yellow-500 self-center" />
                  <div class="flex flex-col">
                    <p class="text-babylon">Refund pending</p>
                    <p class="text-babylon text-xs text-zinc-400">
                      Transfer took too long
                    </p>
                  </div>
                {:else if simpleStatus._tag === "TimeoutSubmitted"}
                  <RotateLeftIcon class="size-6 self-center" />
                  <div class="flex flex-col">
                    <p class="text-babylon">Refunded</p>
                    <p class="text-xs text-zinc-400">Transfer took too long</p>
                  </div>
                {:else}
                  <SharpWarningIcon class="size-6 text-yellow-500 self-center" />
                  <div class="flex flex-col">
                    <p class="text-babylon">Unknown status</p>
                    <p class="text-xs text-zinc-400">{simpleStatus._tag}</p>
                  </div>
                {/if}
              </div>
            </div>
            <section class="flex flex-col px-4">
              <Label>From</Label>
              {#if Option.isSome(sourceChain)}
                {#if settingsStore.showQuoteTokens}
                  <TokenComponent
                    chain={sourceChain.value}
                    denom={transfer.base_token}
                    amount={transfer.base_amount}
                  />
                {/if}
                {#if Option.isSome(sourceChain)}
                  <ChainComponent chain={sourceChain.value} />
                  <AddressComponent
                    address={transfer.sender_display}
                    chain={sourceChain.value}
                    class="text-zinc-400"
                  />
                {:else}
                  <div>{transfer.source_chain.chain_id}</div>
                  <div class="font-mono text-sm text-zinc-400">
                    {transfer.sender_canonical}
                  </div>
                {/if}
              {/if}
              <DateTimeComponent
                class="text-sm text-zinc-400"
                value={transfer.transfer_send_timestamp}
                showSeconds={false}
              />
            </section>

            <section class="flex flex-col px-4">
              <Label>To</Label>
              {#if settingsStore.showQuoteTokens && Option.isSome(destChain)}
                <TokenComponent
                  chain={destChain.value}
                  denom={transfer.quote_token}
                  amount={transfer.quote_amount}
                />
              {/if}
              {#if Option.isSome(destChain)}
                <ChainComponent chain={destChain.value} />
                <AddressComponent
                  address={transfer.receiver_display}
                  chain={destChain.value}
                  class="text-zinc-400"
                />
              {:else}
                <div>{transfer.destination_chain.chain_id}</div>
                <div class="font-mono text-sm text-zinc-400">
                  {transfer.receiver_canonical}
                </div>
              {/if}
              {#if Option.isSome(transfer.transfer_recv_timestamp)}
                <DateTimeComponent
                  class="text-sm text-zinc-400"
                  value={transfer.transfer_recv_timestamp.value}
                  showSeconds={false}
                />
              {/if}
            </section>

            {#if Option.isSome(destChain) && destChain.value.rpc_type === "cosmos"}
              <section class="px-4">
                <Label>
                  Add to wallet
                </Label>
                <p class="text-sm">
                  First time transferring this asset? Make sure you have <b>Keplr</b> connected and
                  <Button
                    variant="inline"
                    onclick={() =>
                    suggestTokenToWallet(
                      destChain.value.chain_id,
                      transfer.quote_token,
                    )}
                  >
                    add it to your Keplr wallet.
                  </Button>
                </p>
                <p class="text-xs mt-4 text-zinc-300 italic">
                  If you do not see a popup, you have either not connected Keplr or already added it
                  to your wallet. In Keplr scroll down to "Manage Asset List" and re-enable the
                  token. Leap does not currently support this feature, but we are working with them
                  to add support soon.
                </p>
              </section>
            {/if}
            {#if Option.isSome(sourceChain)}
              {@const settlement = settlementDelays[sourceChain.value.universal_chain_id]}
              {#if settlement}
                <section class="flex flex-col px-4">
                  <Label>ETA</Label>
                  <p class="text-sm">
                    {sourceChain.value.display_name} is an L2. Outbound transfers are processed as
                    soon as
                    {sourceChain.value.display_name} settles (<A
                      class="underline"
                      href={settlement.url}
                    >happens every {settlement.interval}</A>).
                  </p>
                </section>
              {/if}
              {@const finality = finalityDelays[sourceChain.value.universal_chain_id]}
              {#if finality}
                <section class="flex flex-col px-4">
                  <Label>ETA</Label>
                  <p class="text-sm">
                    Transfers out of {sourceChain.value.display_name} are processed as soon as
                    {sourceChain.value.display_name} reaches finality. (<A
                      class="underline"
                      href={finality.url}
                    >approximately {finality.interval}</A>).
                  </p>
                </section>
              {/if}
            {/if}
          </div>

          <PacketTracesComponent
            packetTraces={transfer.traces}
            showAcks={false}
            mode="transfer"
          />
        </div>
      {/if}
    </div>
  </Card>

  <!-- Packet Details Card with toggle button -->
  <div>
    <button
      class="flex items-center justify-center w-full gap-2 py-2 px-4 text-left hover:text-zinc-300 text-zinc-400 cursor-pointer transition-colors"
      onclick={() => showPacketDetails = !showPacketDetails}
    >
      <span>Packet Details</span>
      <span
        class="transition-transform duration-300"
        style={showPacketDetails ? "transform: rotate(180deg)" : ""}
      >↓</span>
    </button>

    {#if showPacketDetails}
      <Card
        divided
        transition={false}
      >
        <PacketComponent />
      </Card>
    {/if}
  </div>
</Sections>

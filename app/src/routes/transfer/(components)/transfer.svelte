<script lang="ts">
import {
  switchChain,
  writeContract,
  simulateContract,
  getConnectorClient,
  waitForTransactionReceipt
} from "@wagmi/core"
import {
  cosmosHttp,
  createPfmMemo,
  truncateAddress,
  bytesToBech32Address,
  createCosmosSdkClient,
  type TransactionResponse,
  type TransferAssetsParameters,
  bech32ToBech32Address
} from "@union/client"
import { onMount } from "svelte"
import { page } from "$app/stores"
import { toast } from "svelte-sonner"
import { goto } from "$app/navigation"
import Chevron from "./chevron.svelte"
import { useMachine } from "@xstate/svelte"
import { ucs01abi } from "$lib/abi/ucs-01.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import { raise, sleep } from "$lib/utilities/index.ts"
import type { OfflineSigner } from "@leapwallet/types"
import { userBalancesQuery } from "$lib/queries/balance"
import { transferStateMachine } from "../state-machine.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import { userAddrOnChain } from "$lib/utilities/address.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { createBrowserInspector } from "@statelyai/inspect"
import WalletGate from "$lib/components/wallet-gate.svelte"
import { Button } from "$lib/components/ui/button/index.ts"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import CardSectionHeading from "./card-section-heading.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { sepoliaStore, wagmiConfig, evmConnect } from "$lib/wallet/evm"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import { sepolia, berachainTestnetbArtio, arbitrumSepolia } from "viem/chains"
import { get, derived, writable, type Writable, type Readable } from "svelte/store"
import { custom, erc20Abi, parseUnits, getAddress, formatUnits, type Address } from "viem"
import ChainButton from "./chain-button.svelte"
import ChainDialog from "./chain-dialog.svelte"
import AssetsDialog from "./assets-dialog.svelte"
import { truncate } from "$lib/utilities/format.ts"
import DevTools from "$lib/components/dev-tools.svelte"

export let connected: boolean
export let chains: Array<Chain>
export let userAddresses: UserAddresses

const { inspect, ...inspector } = createBrowserInspector({
  autoStart: true
  // import.meta.env.MODE === "development" &&
  // import.meta.env.DEBUG_XSTATE === "true"
})

const { snapshot, send } = useMachine(transferStateMachine, {
  inspect,
  input: {
    chains: chains,
    cosmosStore: $cosmosStore,
    sepoliaStore: $sepoliaStore
  }
})

function swapChainsClick() {
  const fromChain = $snapshot.context["SOURCE_CHAIN_ID"]
  const toChain = $snapshot.context["DESTINATION_CHAIN_ID"]
  const network = chains.find(c => c.chain_id === toChain)?.rpc_type
  if (!(network && fromChain && toChain)) return
  send({ type: "SET_DESTINATION_CHAIN", value: fromChain })
  send({ type: "SET_SOURCE_CHAIN", value: { network, chainId: toChain } })
}

let [dialogOpenFromChain, dialogOpenToChain, dialogOpenToken] = [false, false, false]

// $: console.info($snapshot.context)

$: recipient = $snapshot.context?.["RECIPIENT"] ?? ""
$: sourceChain = chains.find(({ chain_id }) => chain_id === $snapshot.context["SOURCE_CHAIN_ID"])

let _assetBalances = userBalancesQuery({ chains, userAddresses, connected })
let assetBalances = derived(_assetBalances, $_assetBalances => {
  const chainIndex = chains.findIndex(({ chain_id }) => chain_id === sourceChain?.chain_id)
  return $_assetBalances[chainIndex]?.data ?? []
})

let balanceCoversAmount = false

$: buttonText =
  $snapshot.context["ASSET_SYMBOL"] && $snapshot.context["AMOUNT"]
    ? balanceCoversAmount
      ? "transfer"
      : "insufficient balance"
    : $snapshot.context["ASSET_SYMBOL"] && !$snapshot.context["AMOUNT"]
      ? "enter amount"
      : "select asset and enter amount"

$: disableRecipientField =
  $snapshot.context["SOURCE_CHAIN_ID"] === undefined ||
  $snapshot.context["DESTINATION_CHAIN_ID"] === undefined

$: buttonDisabled = !(
  $snapshot.context["AMOUNT"] &&
  $snapshot.context["ASSET_SYMBOL"] &&
  $snapshot.context["DESTINATION_CHAIN_ID"] &&
  $snapshot.context["RECIPIENT"] &&
  $snapshot.context["SOURCE_CHAIN_ID"] &&
  $snapshot.context["AMOUNT"] &&
  balanceCoversAmount
)

let ANIMATION_STATE: "FLIP" | "FLIPPED" | "UNFLIP" | "UNFLIPPED" = "UNFLIPPED"

async function transferASS() {
  let client = $snapshot.context["client"]
  if (!client) send({ type: "SET_CLIENT" })
  await sleep(1_000)
  client = $snapshot.context["client"]

  const payload = $snapshot.context["PAYLOAD"]
  if (!payload) return toast.error("No payload found")

  const transactionPayload = {
    path: payload["path"],
    memo: payload["memo"],
    amount: payload["amount"],
    network: payload["network"],
    recipient: payload["recipient"],
    denomAddress: payload["denomAddress"],
    sourceChannel: payload["sourceChannel"],
    relayContractAddress: payload["relayContractAddress"]
  } satisfies TransferAssetsParameters

  client?.simulateTransaction(transactionPayload)
}
</script>

<DevTools>
  <pre class="text-left w-full">
    {JSON.stringify($snapshot.context, null, 2)}
  </pre>
</DevTools>

<div
  class={cn(
    "size-full duration-1000 transition-colors dark:bg-muted",
    $snapshot.matches("START") ? "bg-black/60" : ""
  )}
></div>

<div class="cube-scene" id="scene">
  <div
    class={cn(
      "cube ",
      !$snapshot.matches("START") ? "cube--flipped" : "no-transition"
    )}
  >
    <div
      class="cube-right font-bold flex items-center justify-center text-xl font-supermolot"
    >
      UNION TESTNET
    </div>
    <Card.Root
      class={cn($snapshot.matches("START") ? "no-transition" : "cube-front")}
    >
      <Card.Header>
        <Card.Title>Transfer</Card.Title>
      </Card.Header>
      <Card.Content class={cn("flex flex-col gap-4")}>
        <section>
          <CardSectionHeading>From</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenFromChain}>
            {$snapshot.context["SOURCE_CHAIN_ID"] ?? "Select chain"}
          </ChainButton>
          <div class="flex flex-col items-center pt-4 -mb-6">
            <Button on:click={swapChainsClick} size="icon" variant="outline">
              <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
            </Button>
          </div>
          <CardSectionHeading>To</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenToChain}>
            {$snapshot.context["DESTINATION_CHAIN_ID"] ?? "Select chain"}
          </ChainButton>
        </section>
        <section>
          <CardSectionHeading>Asset</CardSectionHeading>
          <!-- {#if $sendableBalances !== undefined && $fromChainId}
            {#if $sendableBalances === null}
              Failed to load sendable balances for <b
                >{$fromChain?.display_name}</b
              >.
            {:else if $sendableBalances && $sendableBalances.length === 0}
              You don't have sendable assets on <b>{$fromChain?.display_name}</b
              >. You can get some from
              <a class="underline font-bold" href="/faucet">the faucet</a>
            {:else} -->
          {#if $assetBalances.length > 0}
            <Button
              class="w-full"
              variant="outline"
              on:click={() => (dialogOpenToken = !dialogOpenToken)}
            >
              <div class="flex-1 text-left font-bold text-md">
                {truncate(
                  $snapshot.context["ASSET_SYMBOL"] ?? "Select Asset",
                  12
                )}
              </div>

              <Chevron />
            </Button>
          {:else}
            <a class="underline font-bold" href="/faucet">the faucet</a>
          {/if}
          <!-- 
            {/if}
          {:else}
            Select a chain to send from.
          {/if}
          {#if $assetSymbol !== "" && $sendableBalances !== null && $asset?.address}
            <div class="mt-4 text-xs text-muted-foreground">
              <b
                >{truncate(
                  supportedAsset
                    ? supportedAsset?.display_symbol
                    : $assetSymbol,
                  12
                )}</b
              >
              balance on
              <b>{$fromChain?.display_name}</b> is
              {formatUnits(
                BigInt($asset.balance),
                supportedAsset?.decimals ?? 0
              )}
            </div>
          {/if} -->
        </section>
        <section>
          <CardSectionHeading>Amount</CardSectionHeading>
          <Input
            minlength={1}
            type="number"
            maxlength={64}
            disabled={false}
            autocorrect="off"
            spellcheck="false"
            autocomplete="off"
            placeholder="0.00"
            inputmode="decimal"
            autocapitalize="none"
            class={cn()}
            pattern="^[0-9]*[.,]?[0-9]*$"
            on:input={event => {
              // @ts-expect-error
              const value = event.target?.value
              send({ type: "SET_AMOUNT", value: value })
            }}
          />
        </section>
        <section>
          <CardSectionHeading>Recipient</CardSectionHeading>
          <div class="flex items-start gap-2">
            <div class="w-full">
              <div class="relative w-full mb-2">
                <Input
                  type="text"
                  id="address"
                  readonly={true}
                  required={true}
                  value={recipient}
                  autocorrect="off"
                  spellcheck="false"
                  autocomplete="off"
                  autocapitalize="none"
                  class="disabled:bg-black/30"
                  placeholder="Recipient address"
                  disabled={disableRecipientField}
                  on:input={event => {
                    // @ts-expect-error
                    const value = `${event.target?.value}`.trim()
                    send({ value: value, type: "SET_RECIPIENT" })
                  }}
                />
              </div>
              <div class="flex justify-between px-1">
                {#if $snapshot.context?.["RECIPIENT"]}
                  <button
                    type="button"
                    on:click={() => send({ type: "SET_RECIPIENT", value: "" })}
                    class="text-xs text-muted-foreground hover:text-primary transition"
                  >
                    Reset
                  </button>
                {/if}
              </div>
            </div>
          </div>
        </section>
      </Card.Content>
      <Card.Footer class="flex flex-col gap-4 items-start">
        <Button
          type="button"
          disabled={buttonDisabled}
          on:click={async event => {
            event.preventDefault()
            ANIMATION_STATE = "FLIP"
            await sleep(500)
            let client = $snapshot.context["client"]
            if (!client) send({ type: "SET_CLIENT" })
            await sleep(1_000)
            client = $snapshot.context["client"]
            // client

            // transferState.set({ kind: "FLIPPING" })
            // await sleep(1200)
            // transfer()
          }}
        >
          {buttonText}
        </Button>
      </Card.Footer>
    </Card.Root>

    <!-- {#if $transferState.kind !== "PRE_TRANSFER"}
      <Card.Root class={cn("cube-back p-6")}>
        {#if $fromChain}
          <Stepper
            steps={stepperSteps}
            on:cancel={() => transferState.set({ kind: "PRE_TRANSFER" })}
            onRetry={() => {
              transferState.update(ts => {
                // @ts-ignore
                ts.error = undefined
                return ts
              })
              transfer()
            }}
          />
        {/if}
      </Card.Root>
      <div
        class="cube-left font-bold flex items-center justify-center text-xl font-supermolot"
      >
        UNION TESTNET
      </div>
    {/if} -->
  </div>
</div>

<ChainDialog
  {connected}
  kind="from"
  userAddr={userAddresses}
  bind:dialogOpen={dialogOpenFromChain}
  chains={chains.filter(c => c.enabled_staging)}
  selectedChain={`${$snapshot.context["SOURCE_CHAIN_ID"]}`}
  onChainSelect={newSelectedChain => {
    const network = chains.find(c => c.chain_id === newSelectedChain)?.rpc_type
    if (!network) return
    send({
      type: "SET_SOURCE_CHAIN",
      value: { network, chainId: newSelectedChain }
    })
  }}
/>

<ChainDialog
  kind="to"
  {connected}
  userAddr={userAddresses}
  bind:dialogOpen={dialogOpenToChain}
  chains={chains.filter(c => c.enabled_staging)}
  selectedChain={`${$snapshot.context["DESTINATION_CHAIN_ID"]}`}
  onChainSelect={newSelectedChain => {
    const network = chains.find(c => c.chain_id === newSelectedChain)?.rpc_type
    const prefix = chains.find(
      c => c.chain_id === newSelectedChain
    )?.addr_prefix
    const recipientAddress =
      network === "cosmos"
        ? $cosmosStore.address && prefix
          ? bech32ToBech32Address({
              toPrefix: prefix,
              address: $cosmosStore.address
            })
          : ""
        : network === "evm"
          ? $sepoliaStore.address
          : $snapshot.context["RECIPIENT"]
    if (recipientAddress) {
      send({ type: "SET_RECIPIENT", value: recipientAddress })
    }
    send({
      value: newSelectedChain,
      type: "SET_DESTINATION_CHAIN"
    })
  }}
/>

{#if sourceChain && $assetBalances.length > 0}
  <AssetsDialog
    chain={sourceChain}
    assets={$assetBalances}
    bind:dialogOpen={dialogOpenToken}
    onAssetSelect={asset => {
      console.log("Selected Asset: ", asset)
      send({
        type: "SET_ASSET",
        value: { symbol: asset.symbol, denomAddress: asset.address }
      })
    }}
  />
{/if}

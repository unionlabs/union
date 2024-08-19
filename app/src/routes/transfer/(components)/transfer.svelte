<script lang="ts">
import {
  cosmosHttp,
  createPfmMemo,
  truncateAddress,
  createUnionClient,
  bech32AddressToHex,
  bytesToBech32Address,
  bech32ToBech32Address,
  type TransactionResponse,
  type CosmosClientParameters,
  type TransferAssetsParameters
} from "@union/client"
import { onMount } from "svelte"
import { page } from "$app/stores"
import { toast } from "svelte-sonner"
import Chevron from "./chevron.svelte"
import { useMachine } from "@xstate/svelte"
import { ucs01abi } from "$lib/abi/ucs-01.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import { userAddrEvm } from "$lib/wallet/evm"
import ChainButton from "./chain-button.svelte"
import ChainDialog from "./chain-dialog.svelte"
import { cosmosStore } from "$lib/wallet/cosmos"
import AssetsDialog from "./assets-dialog.svelte"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { truncate } from "$lib/utilities/format.ts"
import { raise, sleep } from "$lib/utilities/index.ts"
import DevTools from "$lib/components/dev-tools.svelte"
import { userBalancesQuery } from "$lib/queries/balance"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import { userAddrOnChain } from "$lib/utilities/address.ts"
import { createBrowserInspector } from "@statelyai/inspect"
import { Button } from "$lib/components/ui/button/index.ts"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import CardSectionHeading from "./card-section-heading.svelte"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { getCosmosOfflineSigner } from "$lib/wallet/cosmos/config.ts"
import type { ChainsQueryResult } from "$lib/graphql/documents/chains"
import { sepoliaStore, wagmiConfig, evmConnect } from "$lib/wallet/evm"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import { sepolia, berachainTestnetbArtio, arbitrumSepolia } from "viem/chains"
import { get, derived, writable, type Writable, type Readable } from "svelte/store"
import { transferStateMachine, transferAnimationMachine } from "../state-machine.ts"
import { getChainId, switchChain, getWalletClient, getConnectorClient } from "@wagmi/core"
import { custom, erc20Abi, parseUnits, getAddress, formatUnits, type Address, http } from "viem"

export let chains: Array<Chain>

const { inspect, ...inspector } = createBrowserInspector({
  autoStart: true
})

const { snapshot, send } = useMachine(transferStateMachine, {
  // inspect,
  input: {
    chains,
    cosmosStore: $cosmosStore,
    sepoliaStore: $sepoliaStore
  }
})

const { snapshot: animationSnapshot, send: animationSend } = useMachine(transferAnimationMachine, {
  inspect,
  input: { defaultDelay: 1_000 }
})

let [dialogOpenFromChain, dialogOpenToChain, dialogOpenToken] = [false, false, false]

$: network = $snapshot.context["NETWORK"]
$: recipient = $snapshot.context?.["RECIPIENT"]

$: sourceChainId = $snapshot.context["SOURCE_CHAIN_ID"]
$: sourceChain = chains.find(({ chain_id }) => chain_id === sourceChainId)
$: destinationChainId = $snapshot.context["DESTINATION_CHAIN_ID"]
$: destinationChain = chains.find(({ chain_id }) => chain_id === destinationChainId)

$: sourceChannel = $snapshot.context["SOURCE_CHANNEL"]
$: relayContractAddress = $snapshot.context["RELAY_CONTRACT_ADDRESS"]
$: denomAddress = $snapshot.context["ASSET"]?.denom
$: assetSymbol = $snapshot.context["ASSET"]?.display_symbol

$: selectedAsset = sourceChain?.assets.find(asset => asset.denom === denomAddress)

$: pfmTransfer =
  sourceChainId &&
  sourceChainId !== "union-testnet-8" &&
  destinationChainId &&
  destinationChainId !== "union-testnet-8"

$: path = (
  pfmTransfer
    ? [`${sourceChainId}`, "union-testnet-8"]
    : [`${sourceChainId}`, `${destinationChainId}`]
) satisfies [string, string]

$: ucsConfiguration = pfmTransfer
  ? sourceChain?.ucs1_configurations["union-testnet-8"]
  : destinationChainId
    ? sourceChain?.ucs1_configurations[destinationChainId]
    : undefined

$: forward =
  pfmTransfer && destinationChainId
    ? sourceChain?.ucs1_configurations["union-testnet-8"].forward?.[destinationChainId]
    : undefined

$: if (sourceChainId && destinationChainId && ucsConfiguration) {
  send({ type: "SET_SOURCE_CHANNEL", value: ucsConfiguration.channel_id })
  send({ type: "SET_RELAY_CONTRACT_ADDRESS", value: ucsConfiguration.contract_address })
}

$: userAddress = derived(
  [userAddrEvm, userAddrCosmos],
  ([$userAddrEvm, $userAddrCosmos]) =>
    ({
      evm: $userAddrEvm,
      cosmos: $userAddrCosmos
    }) as UserAddresses
)

$: cosmosOfflineSigner = sourceChainId ? getCosmosOfflineSigner(sourceChainId) : undefined

$: _allSourceChainAssetBalances = userBalancesQuery({
  chains,
  // @ts-expect-error
  userAddresses: { evm: $userAddrEvm, cosmos: $userAddrCosmos },
  connected:
    $cosmosStore.connectionStatus === "connected" || $sepoliaStore.connectionStatus === "connected"
})

$: allSourceChainAssetBalances = derived(
  _allSourceChainAssetBalances,
  $_allSourceChainAssetBalances => {
    const chainIndex = chains.findIndex(({ chain_id }) => chain_id === sourceChain?.chain_id)
    return $_allSourceChainAssetBalances[chainIndex]?.data ?? []
  }
)

$: assetBalance = derived(allSourceChainAssetBalances, $allSourceChainAssetBalances => {
  const asset = $allSourceChainAssetBalances.find(asset => asset.address === denomAddress)
  if (!(asset && selectedAsset)) return
  return {
    address: asset.address,
    balance: asset.balance,
    name: selectedAsset.display_name,
    symbol: selectedAsset.display_symbol,
    formattedBalance: formatUnits(asset.balance, selectedAsset.decimals)
  }
})

$: console.info($assetBalance)

let inputAmount = ""
$: inputAmount = inputAmount.replaceAll(/[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g, "")
$: Number.parseFloat(inputAmount) >= 0 && send({ type: "SET_AMOUNT", value: BigInt(inputAmount) })
$: amount = $snapshot.context["AMOUNT"]

$: memo = pfmTransfer
  ? createPfmMemo({
      receiver:
        destinationChain?.rpc_type === "evm"
          ? `${recipient}`
          : bech32AddressToHex({ address: `${recipient}` }),
      port: `${forward?.port}`,
      channel: `${forward?.channel_id}`
    })
  : `transferring ${inputAmount} ${assetSymbol}`

/**
 * TODO: this doesn't take gas and fees in general into account
 */
$: balanceCoversAmount = derived(assetBalance, $assetBalance => {
  if (!(inputAmount && $assetBalance?.balance && selectedAsset?.decimals)) return false
  const amount = parseUnits(inputAmount, selectedAsset?.decimals)
  return amount <= $assetBalance?.balance
})

// $: console.info(
//   JSON.stringify(
//     {
//       $balanceCoversAmount,
//       amount,
//       formattedAmount: amount ? formatUnits(amount, selectedAsset?.decimals ?? 0) : 0n,
//       balance: $assetBalance?.balance,
//       formattedBalance: $assetBalance?.formattedBalance,
//     },
//     undefined,
//     2,
//   ),
// )

$: buttonText =
  denomAddress && inputAmount
    ? $balanceCoversAmount
      ? destinationChainId
        ? "transfer"
        : "select destination chain"
      : "insufficient balance"
    : denomAddress && !inputAmount
      ? "enter amount"
      : "select asset and enter amount"

$: disableRecipientField = sourceChainId === undefined || destinationChainId === undefined

$: buttonDisabled = !(
  inputAmount &&
  denomAddress &&
  sourceChainId &&
  destinationChainId &&
  recipient &&
  $balanceCoversAmount
)

function swapChainsClick(_event: MouseEvent) {
  const [fromChain, toChain] = [sourceChainId, destinationChainId]
  const network = chains.find(chain => chain.chain_id === toChain)?.rpc_type
  if (!(network && fromChain && toChain)) return
  send({ type: "SET_DESTINATION_CHAIN", value: fromChain })
  send({ type: "SET_SOURCE_CHAIN", value: { chainId: toChain, network } })
}

async function onTransferClick(event: MouseEvent) {
  event.preventDefault()

  animationSend({
    delay: 4_000,
    type: "advance",
    target: "FLIPPING"
  })
  console.info($animationSnapshot.context)
  // animationSend({
  //   type: 'nextStep',
  //   value: {
  //     state: 'SWITCHING_TO_CHAIN',
  //   },
  // })

  const currentChainId = getChainId(wagmiConfig)
  if (network === "evm" && currentChainId !== Number(sourceChainId)) {
    // @ts-expect-error
    await switchChain(wagmiConfig, { chainId: Number(sourceChainId) })
  }

  const params = [
    ["network", network],
    ["sourceChainId", sourceChainId],
    ["destinationChainId", destinationChainId],
    ["path", ...path],
    ["relayContractAddress", relayContractAddress],
    ["sourceChannel", sourceChannel],
    ["inputAmount", inputAmount],
    ["recipient", recipient],
    ["denomAddress", denomAddress]
  ]
  params.forEach(([param, value]) => {
    if (!value) return toast.error(`Missing parameter: ${param} with value ${value}`)
  })

  if (
    !(
      network &&
      sourceChainId &&
      destinationChainId &&
      path &&
      relayContractAddress &&
      sourceChannel &&
      inputAmount &&
      recipient &&
      denomAddress
    )
  ) {
    return toast.error("Missing parameters")
  }
  await sleep(1_000)

  const walletClient = await getWalletClient(wagmiConfig, {
    chainId: Number(sourceChainId)
  })

  // const cosmosRpcURLs =
  //   network === 'cosmos'
  //     ? sourceChain?.rpcs?.filter(rpc => rpc.type === 'rpc').map(rpc => rpc.url)
  //     : undefined

  // const cosmosGasDenom =
  //   network === 'cosmos' ? sourceChain?.assets.find(asset => asset.gas_token)?.denom : undefined

  // const cosmos = {
  //   account: cosmosOfflineSigner,
  //   gasPrice: { inputAmount: '0.0025', denom: cosmosGasDenom },
  //   transport: cosmosRpcURLs?.map(url => cosmosHttp(url)),
  // } satisfies CosmosClientParameters

  const evmChain = wagmiConfig.chains.find(chain => chain.id === Number(sourceChainId)) ?? sepolia
  console.info({
    account: cosmosOfflineSigner,
    gasPrice: {
      inputAmount: "0.0025",
      denom: sourceChain?.assets.find(asset => asset.gas_token)?.denom
    },
    transport: cosmosHttp(`https://${sourceChain?.rpcs?.find(rpc => rpc.type === "rpc")?.url}`)
  })
  const client = createUnionClient({
    evm: {
      chain: evmChain,
      account: walletClient.account,
      transport: custom(window.ethereum)
    },
    cosmos: {
      account: cosmosOfflineSigner,
      gasPrice: {
        inputAmount: "0.0025",
        // @ts-expect-error
        denom: sourceChain?.assets.find(asset => asset.gas_token)?.denom
      },
      transport: cosmosHttp(`https://${sourceChain?.rpcs?.find(rpc => rpc.type === "rpc")?.url}`)
    }
  })

  animationSend({
    delay: 2_000,
    type: "advance",
    target: "APPROVING_ASSET"
  })
  const transferAssetsParameters = {
    memo,
    path,
    network,
    recipient,
    denomAddress,
    sourceChannel,
    approve: false,
    relayContractAddress,
    amount: BigInt(inputAmount)
  } satisfies TransferAssetsParameters

  console.info(JSON.stringify(transferAssetsParameters, undefined, 2))

  // @ts-expect-errorq
  const approvalResponse = await client.approveTransaction(transferAssetsParameters)

  animationSend({
    delay: 2_000,
    type: "advance",
    target: "AWAITING_APPROVAL_RECEIPT",
    data: approvalResponse.success ? approvalResponse.data : undefined,
    error: approvalResponse.success ? undefined : new Error(approvalResponse.data)
  })
  const transfer = await client.transferAsset(transferAssetsParameters)

  console.info(transfer)
}

$: _animationSnapshot = $animationSnapshot["context"]
$: console.info(JSON.stringify(_animationSnapshot, undefined, 2))
</script>

<DevTools>
  <pre class="text-left w-full">
    {JSON.stringify(
      {
        sourceChain: $snapshot.context['SOURCE_CHAIN_ID'],
        destinationChain: $snapshot.context['DESTINATION_CHAIN_ID'],
        network: $snapshot.context['NETWORK'],
        assetSymbol: $snapshot.context['ASSET_SYMBOL'],
        assetContract: $snapshot.context['ASSET_DENOM_ADDRESS'],
        inputAmount: $snapshot.context['AMOUNT'],
        recipient: $snapshot.context['RECIPIENT'],
        relayContractAddress: $snapshot.context['RELAY_CONTRACT_ADDRESS'],
        sourceChannel: $snapshot.context['SOURCE_CHANNEL'],
      },
      null,
      2,
    )}
  </pre>
</DevTools>

<div
  class={cn(
    'size-full duration-1000 transition-colors dark:bg-muted',
    $snapshot.matches('START') ? 'bg-black/60' : '',
  )}
></div>
<Button
  on:click={() => {
    animationSend({
      type: 'advance',
      delay: 4_000,
    })
    // console.info(DELAY, LAST_EVENT, ANIMATION_STATE, ANIMATION_STEP)
    animationSend({
      type: 'advance',
    })
    // console.info(DELAY, LAST_EVENT, ANIMATION_STATE, ANIMATION_STEP)
  }}
>
  b
</Button>
<div class="cube-scene" id="scene">
  <div class={cn('cube ', !$snapshot.matches('START') ? 'cube--flipped' : 'no-transition')}>
    <div class="cube-right font-bold flex items-center justify-center text-xl font-supermolot">
      UNION TESTNET
    </div>
    <Card.Root class={cn($snapshot.matches('START') ? 'no-transition' : 'cube-front')}>
      <Card.Header>
        <Card.Title>Transfer</Card.Title>
      </Card.Header>
      <Card.Content class={cn('flex flex-col gap-4')}>
        <section>
          <CardSectionHeading>From</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenFromChain}>
            {$snapshot.context['SOURCE_CHAIN_ID'] ?? 'Select chain'}
          </ChainButton>
          <div class="flex flex-col items-center pt-4 -mb-6">
            <Button on:click={swapChainsClick} size="icon" variant="outline">
              <ArrowLeftRight class="size-5 dark:text-white rotate-90" />
            </Button>
          </div>
          <CardSectionHeading>To</CardSectionHeading>
          <ChainButton bind:dialogOpen={dialogOpenToChain}>
            {$snapshot.context['DESTINATION_CHAIN_ID'] ?? 'Select chain'}
          </ChainButton>
        </section>
        <section>
          <CardSectionHeading>Asset</CardSectionHeading>
          {#if !$snapshot.context['SOURCE_CHAIN_ID']}
            Select a chain to send from.
          {:else if $allSourceChainAssetBalances.length > 0 && $snapshot.context['SOURCE_CHAIN_ID'] && $snapshot.context['SOURCE_CHAIN_ID'].length > 0}
            {@const assetName = $snapshot.context['ASSET']?.display_symbol}
            <Button
              class="w-full"
              variant="outline"
              on:click={() => (dialogOpenToken = !dialogOpenToken)}
            >
              <div class="flex-1 text-left font-bold text-md">
                {truncate(assetName ?? 'Select Asset', 12)}
              </div>

              <Chevron />
            </Button>
          {:else}
            You don't have sendable assets on <b>{sourceChainId}</b>. You can get some from
            <a class="underline font-bold" href="/faucet">the faucet</a>
          {/if}
          {#if denomAddress && assetSymbol && $assetBalance?.balance}
            <div class="mt-4 text-xs text-muted-foreground">
              <b>{truncate(assetSymbol, 12)}</b>
              balance on
              <b>{sourceChain?.display_name}</b> is
              {$assetBalance?.formattedBalance}
            </div>
          {/if}
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
            bind:value={inputAmount}
            autocapitalize="none"
            class={cn(
              inputAmount && !$balanceCoversAmount ? 'border-red-500' : '',
              'focus:ring-0 focus-visible:ring-0 disabled:bg-black/30',
            )}
            pattern="^[0-9]*[.,]?[0-9]*$"
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
                    send({ value: `${event.target?.value}`.trim(), type: 'SET_RECIPIENT' })
                  }}
                />
              </div>
              <div class="flex justify-between px-1">
                {#if $snapshot.context?.['RECIPIENT']}
                  <button
                    type="button"
                    on:click={() => send({ type: 'SET_RECIPIENT', value: '' })}
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
        <Button type="button" disabled={buttonDisabled} on:click={onTransferClick}>
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
  kind="from"
  userAddr={$userAddress ?? null}
  bind:dialogOpen={dialogOpenFromChain}
  chains={chains.filter(c => c.enabled_staging)}
  selectedChain={sourceChain?.display_name ?? ''}
  onChainSelect={newSelectedChain => {
    const network = chains.find(c => c.chain_id === newSelectedChain)?.rpc_type
    if (!network) return
    send({
      type: 'SET_SOURCE_CHAIN',
      value: { network, chainId: newSelectedChain },
    })
  }}
/>

<ChainDialog
  kind="to"
  userAddr={$userAddress ?? null}
  bind:dialogOpen={dialogOpenToChain}
  chains={chains.filter(c => c.enabled_staging)}
  selectedChain={destinationChain?.display_name ?? ''}
  onChainSelect={newSelectedChain => {
    const network = chains.find(c => c.chain_id === newSelectedChain)?.rpc_type
    const prefix = chains.find(c => c.chain_id === newSelectedChain)?.addr_prefix
    const recipientAddress =
      network === 'cosmos'
        ? $cosmosStore.address && prefix
          ? bech32ToBech32Address({
              toPrefix: prefix,
              address: $cosmosStore.address,
            })
          : ''
        : network === 'evm'
          ? $sepoliaStore.address
          : $snapshot.context['RECIPIENT']
    if (recipientAddress) {
      send({ type: 'SET_RECIPIENT', value: recipientAddress })
    }
    send({
      value: newSelectedChain,
      type: 'SET_DESTINATION_CHAIN',
    })
  }}
/>

{#if sourceChain && $allSourceChainAssetBalances.length > 0}
  <AssetsDialog
    chain={sourceChain}
    bind:dialogOpen={dialogOpenToken}
    assets={$allSourceChainAssetBalances}
    onAssetSelect={asset => {
      const selectedAsset = sourceChain.assets.find(({ denom }) => denom === asset['address'])
      if (!selectedAsset) return
      send({ type: 'SET_ASSET', value: selectedAsset })
    }}
  />
{/if}

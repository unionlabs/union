<script lang="ts">
  import {
    cosmosHttp,
    createPfmMemo,
    truncateAddress,
    bytesToBech32Address,
    bech32ToBech32Address,
    createUnionClient,
    type TransactionResponse,
    type TransferAssetsParameters,
  } from '@union/client'
  import { onMount } from 'svelte'
  import { page } from '$app/stores'
  import { toast } from 'svelte-sonner'
  import Chevron from './chevron.svelte'
  import { useMachine } from '@xstate/svelte'
  import { ucs01abi } from '$lib/abi/ucs-01.ts'
  import { cn } from '$lib/utilities/shadcn.ts'
  import { userAddrEvm } from '$lib/wallet/evm'
  import ChainButton from './chain-button.svelte'
  import ChainDialog from './chain-dialog.svelte'
  import { cosmosStore } from '$lib/wallet/cosmos'
  import AssetsDialog from './assets-dialog.svelte'
  import { userAddrCosmos } from '$lib/wallet/cosmos'
  import { truncate } from '$lib/utilities/format.ts'
  import { raise, sleep } from '$lib/utilities/index.ts'
  import type { OfflineSigner } from '@leapwallet/types'
  import DevTools from '$lib/components/dev-tools.svelte'
  import { userBalancesQuery } from '$lib/queries/balance'
  import * as Card from '$lib/components/ui/card/index.ts'
  import type { Chain, UserAddresses } from '$lib/types.ts'
  import { Input } from '$lib/components/ui/input/index.ts'
  import { transferStateMachine } from '../state-machine.ts'
  import { userAddrOnChain } from '$lib/utilities/address.ts'
  import { createBrowserInspector } from '@statelyai/inspect'
  import { Button } from '$lib/components/ui/button/index.ts'
  import { getSupportedAsset } from '$lib/utilities/helpers.ts'
  import CardSectionHeading from './card-section-heading.svelte'
  import ArrowLeftRight from 'virtual:icons/lucide/arrow-left-right'
  import { getCosmosChainInfo } from '$lib/wallet/cosmos/chain-info.ts'
  import type { ChainsQueryResult } from '$lib/graphql/documents/chains'
  import { sepoliaStore, wagmiConfig, evmConnect } from '$lib/wallet/evm'
  import { submittedTransfers } from '$lib/stores/submitted-transfers.ts'
  import { sepolia, berachainTestnetbArtio, arbitrumSepolia } from 'viem/chains'
  import { getConnections, getConnectorClient, getWalletClient } from '@wagmi/core'
  import { get, derived, writable, type Writable, type Readable } from 'svelte/store'
  import { custom, erc20Abi, parseUnits, getAddress, formatUnits, type Address, http } from 'viem'

  export let chains: Array<Chain>
  // $: console.info(JSON.stringify({ chains }, undefined, 2))

  const { inspect, ...inspector } = createBrowserInspector({
    autoStart: true,
  })

  const { snapshot, send } = useMachine(transferStateMachine, {
    inspect,
    input: {
      chains,
      cosmosStore: $cosmosStore,
      sepoliaStore: $sepoliaStore,
    },
  })

  function swapChainsClick() {
    const fromChain = $snapshot.context['SOURCE_CHAIN_ID']
    const toChain = $snapshot.context['DESTINATION_CHAIN_ID']
    const network = chains.find(c => c.chain_id === toChain)?.rpc_type
    if (!(network && fromChain && toChain)) return
    send({ type: 'SET_DESTINATION_CHAIN', value: fromChain })
    send({ type: 'SET_SOURCE_CHAIN', value: { chainId: toChain, network } })
  }

  let [dialogOpenFromChain, dialogOpenToChain, dialogOpenToken] = [false, false, false]

  // $: console.info($snapshot.context)

  $: network = $snapshot.context['NETWORK']
  $: recipient = $snapshot.context?.['RECIPIENT']
  // $: console.info(JSON.stringify({ recipient }, undefined, 2))
  $: sourceChainId = $snapshot.context['SOURCE_CHAIN_ID']
  $: sourceChain = chains.find(({ chain_id }) => chain_id === $snapshot.context['SOURCE_CHAIN_ID'])
  $: console.info(JSON.stringify({ sourceChain }, undefined, 2))
  $: destinationChainId = $snapshot.context['DESTINATION_CHAIN_ID']
  $: destinationChain = chains.find(
    ({ chain_id }) => chain_id === $snapshot.context['DESTINATION_CHAIN_ID'],
  )
  $: sourceChannel = $snapshot.context['SOURCE_CHANNEL']
  $: relayContractAddress = $snapshot.context['RELAY_CONTRACT_ADDRESS']
  $: denomAddress = $snapshot.context['ASSET_DENOM_ADDRESS']

  $: {
    if (sourceChainId !== undefined && destinationChainId !== undefined) {
      if (sourceChain?.chain_id === 'union-testnet-8') {
        const ucsConfiguration = sourceChain?.ucs1_configurations[destinationChainId]
        if (ucsConfiguration) {
          send({ type: 'SET_SOURCE_CHANNEL', value: ucsConfiguration.channel_id })
          send({ type: 'SET_RELAY_CONTRACT_ADDRESS', value: ucsConfiguration.contract_address })
        }
      } else if (destinationChain?.chain_id === 'union-testnet-8') {
        const ucsConfiguration = sourceChain?.ucs1_configurations[destinationChainId]
        if (ucsConfiguration) {
          send({ type: 'SET_SOURCE_CHANNEL', value: ucsConfiguration.channel_id })
          send({ type: 'SET_RELAY_CONTRACT_ADDRESS', value: ucsConfiguration.contract_address })
        }
      }
    }
  }

  $: userAddress = derived(
    [userAddrEvm, userAddrCosmos],
    ([$userAddrEvm, $userAddrCosmos]) =>
      ({
        evm: $userAddrEvm,
        cosmos: $userAddrCosmos,
      }) as UserAddresses,
  )

  let _assetBalances = userBalancesQuery({
    chains,
    connected: true,
    // @ts-expect-error
    userAddresses: { evm: $userAddrEvm, cosmos: $userAddrCosmos },
  })

  $: assetBalances = derived(_assetBalances, $_assetBalances => {
    const chainIndex = chains.findIndex(({ chain_id }) => chain_id === sourceChain?.chain_id)
    return $_assetBalances[chainIndex]?.data ?? []
  })

  $: cosmosSigner = (
    sourceChainId
      ? $cosmosStore.connectedWallet === 'keplr'
        ? window.keplr?.getOfflineSigner(sourceChainId, { disableBalanceCheck: false })
        : $cosmosStore.connectedWallet === 'leap'
          ? window.leap?.getOfflineSigner(sourceChainId, { disableBalanceCheck: false })
          : undefined
      : undefined
  ) as OfflineSigner

  let amount = ''
  $: amount = amount.replaceAll(/[^0-9.]|\.(?=\.)|(?<=\.\d+)\./g, '')
  $: Number.parseFloat(amount) > 0 && send({ type: 'SET_AMOUNT', value: BigInt(amount) })

  let balanceCoversAmount = true

  $: buttonText =
    $snapshot.context['ASSET_SYMBOL'] && $snapshot.context['AMOUNT']
      ? balanceCoversAmount
        ? 'transfer'
        : 'insufficient balance'
      : $snapshot.context['ASSET_SYMBOL'] && !$snapshot.context['AMOUNT']
        ? 'enter amount'
        : 'select asset and enter amount'

  $: disableRecipientField =
    $snapshot.context['SOURCE_CHAIN_ID'] === undefined ||
    $snapshot.context['DESTINATION_CHAIN_ID'] === undefined

  $: buttonDisabled = !(
    $snapshot.context['AMOUNT'] &&
    $snapshot.context['ASSET_SYMBOL'] &&
    $snapshot.context['DESTINATION_CHAIN_ID'] &&
    $snapshot.context['RECIPIENT'] &&
    $snapshot.context['SOURCE_CHAIN_ID'] &&
    $snapshot.context['AMOUNT'] &&
    balanceCoversAmount
  )

  let ANIMATION_STATE: 'FLIP' | 'FLIPPED' | 'UNFLIP' | 'UNFLIPPED' = 'UNFLIPPED'

  async function onTransferClick(event: MouseEvent) {
    event.preventDefault()
    // if (
    //   !network ||
    //   !sourceChainId ||
    //   !destinationChainId ||
    //   !relayContractAddress ||
    //   !sourceChannel ||
    //   !amount ||
    //   !recipient ||
    //   !denomAddress
    // ) {
    //   return toast.error('Missing parameters')
    // }
    const params = [
      ['network', network],
      ['sourceChainId', sourceChainId],
      ['destinationChainId', destinationChainId],
      ['relayContractAddress', relayContractAddress],
      ['sourceChannel', sourceChannel],
      ['amount', amount],
      ['recipient', recipient],
      ['denomAddress', denomAddress],
    ]
    params.forEach(([param, value]) => {
      if (!value) return toast.error(`Missing parameter: ${param} with value ${value}`)
    })

    if (
      !network ||
      !sourceChainId ||
      !destinationChainId ||
      !relayContractAddress ||
      !sourceChannel ||
      !amount ||
      !recipient ||
      !denomAddress
    ) {
      return toast.error('Missing parameters')
    }
    await sleep(1_000)

    const walletClient = await getWalletClient(wagmiConfig, {
      chainId: Number(sourceChainId),
    })
    const client = createUnionClient({
      evm: {
        chain: sepolia,
        account: walletClient.account,
        transport: custom(window.ethereum),
      },
      cosmos: {
        account: cosmosSigner,
        gasPrice: { amount: '0.0025', denom: 'muno' },
        transport: cosmosHttp('https://rpc.testnet-8.union.build'),
      },
    })
    const transferAssetsParameters = {
      network,
      recipient,
      denomAddress,
      sourceChannel,
      approve: true,
      relayContractAddress,
      memo: 'Test Transfer',
      amount: BigInt(amount),
      path: [sourceChainId, destinationChainId],
    } satisfies TransferAssetsParameters

    console.info(JSON.stringify(transferAssetsParameters, undefined, 2))
    const transfer = await client.transferAsset(transferAssetsParameters)

    console.info(transfer)
  }

  // $: console.info($snapshot.context)
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
        amount: $snapshot.context['AMOUNT'],
        recipient: $snapshot.context['RECIPIENT'],
        relayContractAddress: $snapshot.context['RELAY_CONTRACT_ADDRESS'],
        sourceChannel: $snapshot.context['SOURCE_CHANNEL'],

        // assetBalances: $assetBalances,
        // userAddress: $userAddress
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
          {:else if $assetBalances.length > 0 && $snapshot.context['SOURCE_CHAIN_ID'] && $snapshot.context['SOURCE_CHAIN_ID'].length > 0}
            <Button
              class="w-full"
              variant="outline"
              on:click={() => (dialogOpenToken = !dialogOpenToken)}
            >
              <div class="flex-1 text-left font-bold text-md">
                {truncate($snapshot.context['ASSET_SYMBOL'] ?? 'Select Asset', 12)}
              </div>

              <Chevron />
            </Button>
          {:else}
            You don't have sendable assets on <b>{''}</b>. You can get some from
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
            bind:value={amount}
            class={cn(
              !balanceCoversAmount && amount ? 'border-red-500' : '',
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
                    const value = `${event.target?.value}`.trim()
                    send({ value: value, type: 'SET_RECIPIENT' })
                    // send({ type: "CONSTRUCT_PAYLOAD" })
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
  selectedChain={`${$snapshot.context['SOURCE_CHAIN_ID']}`}
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
  selectedChain={`${$snapshot.context['DESTINATION_CHAIN_ID']}`}
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

{#if sourceChain && $assetBalances.length > 0}
  <AssetsDialog
    chain={sourceChain}
    assets={$assetBalances}
    bind:dialogOpen={dialogOpenToken}
    onAssetSelect={asset =>
      send({
        type: 'SET_DENOM_ADDRESS_AND_SYMBOL',
        value: { symbol: asset.symbol, denomAddress: asset.address },
      })}
  />
{/if}

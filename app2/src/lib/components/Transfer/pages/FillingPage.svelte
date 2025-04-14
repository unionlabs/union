<script lang="ts">
  import ChainAsset from "$lib/components/Transfer/ChainAsset/index.svelte"
  import Amount from "$lib/components/Transfer/Amount.svelte"
  import Receiver from "$lib/components/Transfer/Receiver.svelte"
  import Button from "$lib/components/ui/Button.svelte"
  import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
  import AddressComponent from "$lib/components/model/AddressComponent.svelte"
  import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
  import { Option, pipe, Data, Match } from "effect"
  import { wallets } from "$lib/stores/wallets.svelte.ts"
  import { Chain } from "@unionlabs/sdk/schema"
  import { uiStore } from "$lib/stores/ui.svelte.ts"
  import {constVoid, flow, identity} from "effect/Function"
  import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
  import type { TransferFlowError } from "$lib/components/Transfer/state/errors.ts"

  type Props = {
    onContinue: () => void
    loading: boolean
    onErrorClose?: () => void
    statusMessage?: string | undefined
    transferErrors?: Option.Option<TransferFlowError> | undefined
  }

  const {
    onContinue,
    loading,
    statusMessage,
    transferErrors = Option.none<TransferFlowError>(),
    onErrorClose = constVoid
  }: Props = $props()


  const buttonText = $derived.by(() => {
    return Option.match(transferErrors, {
      onSome: error =>
        Match.value(error).pipe(
          Match.when(e => e._tag === "InsufficientFundsError", () => "Insufficient funds"),
          Match.when(e => e._tag === "OrderCreationError", () => "Could not create orders"),
          Match.orElse(() => statusMessage)
        ),
      onNone: () => statusMessage ?? "Continue"
    })
  })

  const isButtonEnabled = $derived.by(() =>
    Option.isNone(transferErrors) && !loading
  )

  function handleButtonClick() {
    if (Option.isNone(transferErrors)) {
      onContinue()
    }
  }
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  <div class="flex flex-col gap-4">
    <ChainAsset type="source" />
    <ChainAsset type="destination" />
    <Amount type="source" />
  </div>

  <div class="flex flex-col items-end">
    <div class="flex items-center mr-5 text-zinc-400">
      {#if Option.isSome(transfer.derivedReceiver) && Option.isSome(transfer.destinationChain)}
        <p class="text-xs mb-2">
          <AddressComponent
            truncate
            address={transfer.derivedReceiver.value}
            chain={transfer.destinationChain.value}
          />
        </p>
      {:else}
        <p class="text-xs mb-2">No receiver</p>
      {/if}
      <AngleArrowIcon class="rotate-270" />
    </div>

    <div class="w-full items-end flex gap-2">
      <Button
        class="flex-1"
        variant="primary"
        onclick={handleButtonClick}
        disabled={!isButtonEnabled}
      >
        {buttonText}
      </Button>
      <Receiver />
    </div>
  </div>
</div>

<!--{#if topError && Option.isSome(topError)}-->
<!--  <div class="absolute bottom-0 left-0 right-0">-->
<!--    <ErrorComponent-->
<!--      onClose={onErrorClose}-->
<!--      error={topError.value as unknown as any}-->
<!--    />-->
<!--  </div>-->
<!--{/if}-->

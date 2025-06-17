<script lang="ts">
import AddressInput from "$lib/components/AddressInput.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Input from "$lib/components/ui/Input.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import Amount from "$lib/transfer/shared/components/Amount.svelte"
import ChainAsset from "$lib/transfer/shared/components/ChainAsset/index.svelte"
import FeeDetails from "$lib/transfer/shared/components/FeeDetails.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { ContextFlowError } from "$lib/transfer/shared/errors"
import { AddressCosmosCanonical } from "@unionlabs/sdk/schema"
import { Array as A, Match, Option } from "effect"
import SenderInput from "../components/SenderInput.svelte"

type Props = {
  onContinue: () => void
  loading: boolean
  onErrorClose?: () => void
  statusMessage?: string
  errors?: Option.Option<ContextFlowError>
}

const {
  onContinue,
  loading,
  statusMessage,
  errors = Option.none<ContextFlowError>(),
}: Props = $props()

let isModalOpen = $state(false)
let receiverErrors: string[] = $state([])
let senderErrors: string[] = $state([])
let _errors = $derived(
  A.isEmptyArray(senderErrors) && A.isEmptyArray(receiverErrors),
)
let feeDetailsOpen = $state<boolean>(false)

const uiStatus = $derived.by(() => {
  return Option.match(errors, {
    onSome: error => {
      const match = Match.type<ContextFlowError>().pipe(
        Match.tag("BalanceLookupError", () => ({
          text: "Failed checking balance",
          error,
        })),
        Match.tag("AllowanceCheckError", () => ({
          text: "Failed checking allowance",
          error,
        })),
        Match.tag("OrderCreationError", () => ({
          text: "Could not create orders",
          error,
        })),
        Match.orElse(() => ({
          text: statusMessage ?? "Continue",
          error,
        })),
      )
      return match(error)
    },

    onNone: () => ({
      text: statusMessage ?? "Continue",
      error: null,
    }),
  })
})

const isButtonEnabled = $derived.by(() =>
  !loading && A.isEmptyArray(senderErrors) && A.isEmptyArray(receiverErrors)
)
</script>

<div class="min-w-full flex flex-col grow">
  <div class="flex flex-col gap-4 p-4">
    <AddressInput
      label="sender"
      type="sender"
      chain={transferData.sourceChain}
      address={transferData.raw.sender}
      onValid={(address, encoded) => {
        transferData.raw.updateField("sender", address)
        wallets.addInputAddress(encoded as AddressCosmosCanonical)
        senderErrors = []
      }}
      onError={(xs) => {
        senderErrors = xs
      }}
    />
    {#if A.isNonEmptyReadonlyArray(senderErrors)}
      <ul>
        {#each senderErrors as message}
          <li class="text-red-500 text-xs uppercase">
            {message}
          </li>
        {/each}
      </ul>
    {/if}
    <ChainAsset type="source" />
    <ChainAsset type="destination" />
    <AddressInput
      label="receiver"
      chain={transferData.destinationChain}
      address={transferData.raw.receiver}
      type="receiver"
      onValid={(address) => {
        transferData.raw.updateField("receiver", address)
        receiverErrors = []
      }}
      onError={(xs) => {
        receiverErrors = xs
      }}
    />
    {#if A.isNonEmptyReadonlyArray(receiverErrors)}
      <ul>
        {#each receiverErrors as message}
          <li class="text-red-500 text-xs uppercase">
            {message}
          </li>
        {/each}
      </ul>
    {/if}
    <Amount type="source" />
    <!-- TODO: replace with real fee
    <FeeDetails
      open={feeDetailsOpen}
      onToggle={(newOpen: boolean) => feeDetailsOpen = newOpen}
    />
    -->
  </div>

  <div class="grow"></div>

  <div class="p-4 flex justify-between gap-2 border-t border-zinc-800 sticky bottom-0 bg-zinc-925">
    <div class="w-full items-end flex gap-2">
      {#if Option.isSome(errors)}
        <Button
          class="flex-1"
          variant="danger"
          onclick={() => (isModalOpen = true)}
          disabled={!isButtonEnabled}
        >
          {uiStatus.text}
        </Button>
      {:else}
        <Button
          class="flex-1"
          variant="primary"
          onclick={onContinue}
          disabled={!isButtonEnabled}
        >
          {uiStatus.text}
        </Button>
      {/if}
    </div>
  </div>
</div>

<InsetError
  open={isModalOpen}
  error={uiStatus.error}
  onClose={() => (isModalOpen = false)}
/>

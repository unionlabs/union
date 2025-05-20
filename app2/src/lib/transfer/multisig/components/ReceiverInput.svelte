<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { Bech32 } from "@unionlabs/sdk/schema"
import * as Address from "@unionlabs/sdk/schema/address"
import { Array as A, Either as E, Match, Option as O, ParseResult, pipe, Schema as S } from "effect"
import { apply, constVoid, flow } from "effect/Function"
import { onMount } from "svelte"
import type { FormEventHandler } from "svelte/elements"

let messages = $state.raw<ReadonlyArray<string>>([])
const receiver = $derived(transferData.raw.receiver)

const schemaToUse = $derived(
  O.map(
    transferData.destinationChain,
    flow(
      x => x.rpc_type,
      Match.value,
      Match.when("evm", () => Address.ERC55),
      Match.when("cosmos", () => Bech32),
      Match.orElseAbsurd,
      S.asSchema,
    ),
  ),
)

const validateAddress = (address: string) =>
  pipe(
    schemaToUse,
    O.map(
      flow(
        S.encodeUnknownEither,
        apply(address),
        E.match({
          onLeft: error => {
            messages = pipe(
              error,
              ParseResult.ArrayFormatter.formatErrorSync,
              A.map(x => x.message),
            )
          },
          onRight: () => {
            messages = A.empty()
            transferData.raw.updateField("receiver", address)
          },
        }),
      ),
    ),
    O.getOrElse(constVoid),
  )

const onInput: FormEventHandler<HTMLInputElement> = event =>
  validateAddress(event.currentTarget.value)

onMount(() => {
  if (receiver) {
    validateAddress(receiver)
  }
})
</script>

<Input
  label="receiver"
  id="receiver"
  type="text"
  disabled={O.isNone(schemaToUse)}
  required
  autocorrect="off"
  placeholder="0x00"
  spellcheck="false"
  autocomplete="off"
  inputmode="text"
  autocapitalize="none"
  value={receiver}
  class="h-14 text-center text-lg"
  oninput={onInput}
/>

{#if A.isNonEmptyReadonlyArray(messages)}
  <ul>
    {#each messages as message}
      <li class="text-red-500 text-xs uppercase">
        {message}
      </li>
    {/each}
  </ul>
{/if}

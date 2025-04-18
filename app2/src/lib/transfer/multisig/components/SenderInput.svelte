<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { Bech32FromAddressCanonicalBytesWithPrefix } from "@unionlabs/sdk/schema"
import { Array as A, Either as E, ParseResult, pipe, Schema as S } from "effect"
import { apply } from "effect/Function"
import type { FormEventHandler } from "svelte/elements"
import { onMount } from "svelte"
import { signingMode } from "$lib/transfer/signingMode.svelte.js"

let messages = $state.raw<ReadonlyArray<string>>([])
const sender = $derived(signingMode.mode === "multi" ? transferData.raw.sender : "")

const validateAddress = (address: string) => {
  pipe(
    Bech32FromAddressCanonicalBytesWithPrefix("bbn1"),
    S.encodeUnknownEither,
    apply(address),
    E.match({
      onLeft: error => {
        messages = pipe(
          error,
          ParseResult.ArrayFormatter.formatErrorSync,
          A.map(x => x.message)
        )
      },
      onRight: encoded => {
        messages = A.empty()
        transferData.raw.updateField("sender", address)
        wallets.addInputAddress(encoded)
      }
    })
  )
}

const onInput: FormEventHandler<HTMLInputElement> = event =>
  validateAddress(event.currentTarget.value)

onMount(() => {
  if (sender) {
    validateAddress(sender)
  }
})
</script>

<Input
  label="sender"
  id="sender"
  type="text"
  required
  autocorrect="off"
  placeholder="bbn1"
  spellcheck="false"
  autocomplete="off"
  inputmode="text"
  autocapitalize="none"
  value={sender}
  oninput={onInput}
  class="h-14 text-center text-lg"
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

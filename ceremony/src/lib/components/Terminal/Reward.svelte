<script lang="ts">
import { AddressForm, type ValidState } from "$lib/components/address"
import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { user } from "$lib/state/session.svelte.ts"
import { axiom } from "$lib/utils/axiom.ts"
import { onDestroy, onMount } from "svelte"

let { terminal } = getState()

let showInput = $state(true)

let validation = (val: ValidState) => {
  showInput = val === "INVALID"
}

onMount(() => {
  terminal.setStep(5)
  terminal.updateHistory({
    text: "Add an address, you may receive rewards for successful contributions.",
  })
  terminal.updateHistory({
    text: "You can enter your union or any cosmos address, or type \"skip\".",
  })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_address" }])
})

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if showInput}
  <div class="flex w-full gap-1">
    <div class="whitespace-nowrap">
      <Print>Enter address:</Print>
    </div>
    <AddressForm {validation} />
  </div>
{/if}

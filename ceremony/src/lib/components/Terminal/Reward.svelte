<script lang="ts">
import { AddressForm, type ValidState } from "$lib/components/address"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"

let { terminal } = getState()

let showInput = $state(true)

let validation = (val: ValidState) => {
  showInput = val === "INVALID"
}

onMount(() => {
  terminal.updateHistory({
    text: "Add an address, you may receive rewards for successful contributions."
  })
  terminal.updateHistory({
    text: 'You can enter your union or any cosmos address, or type "skip".'
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
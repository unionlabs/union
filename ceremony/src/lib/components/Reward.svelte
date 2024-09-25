<script lang="ts">
  import {AddressForm, type ValidState} from "$lib/components/address"
  import Print from "$lib/components/TerminalApp/Print.svelte";
  import {getState} from "$lib/state/index.svelte.ts";
  import {onMount} from "svelte";

  let { terminal } = getState()

  let showInput = $state(true)

  let validation = (val: ValidState) => {
    showInput = val === "INVALID";
  }

  onMount(() => {
    terminal.updateHistory("Add an address, you may receive rewards for successful contributions.")
    terminal.updateHistory('You can enter your union or any cosmos address, or type "skip".')
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
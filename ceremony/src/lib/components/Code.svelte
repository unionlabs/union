<script lang="ts">
  import {callJoinQueue} from "$lib/supabase"
  import {toast} from "svelte-sonner"
  import type {Contributor} from "$lib/state/contributor.svelte.ts"
  import Button from "$lib/components/Button.svelte"
  import Print from "$lib/components/TerminalApp/Print.svelte";
  import {getState} from "$lib/state/index.svelte.ts";
  import {onDestroy, onMount} from "svelte";
  import {sleep} from "$lib/utils/utils.ts";

  const {contributor, terminal} = getState()

  let inputCode: string = $state("")
  let normalizedCode = $derived(normalizeString(inputCode))
  let inputElement: HTMLInputElement
  let showInput = $state(true)

  onMount(() => {
    if (inputElement) {
      inputElement.focus();
    }
  });

  function normalizeString(input: string): string {
    return input.toLowerCase().replace(/[^a-z0-9]/gi, "")
  }

  async function handleCodeJoin() {
    try {
      showInput = false
      terminal.updateHistory("Checking code...", {duplicate: true})
      console.log(normalizedCode)
      await sleep(1000)
      const codeOk = await callJoinQueue(normalizedCode)
      if (codeOk) {
        contributor.setAllowanceState("hasRedeemed")
        terminal.updateHistory("Code successfully redeemed")
      } else {
        terminal.updateHistory("The code is not valid", {duplicate: true})
        showInput = true
      }
    } catch (error) {
      console.error("Error redeeming code:", error)
      terminal.updateHistory("An error occurred while redeeming the code")
      showInput = true
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      handleCodeJoin();
    }
  }
</script>

{#if showInput}
  <div class="flex w-full gap-1">
    <div class="whitespace-nowrap">
      <Print>Enter code:</Print>
    </div>
    <input
            autofocus
            bind:this={inputElement}
            bind:value={inputCode}
            onkeydown={handleKeyDown}
            class="inline-flex bg-transparent w-full text-union-accent-500 outline-none focus:ring-0 focus:border-none"
            style="--tw-ring-color: transparent;"
    />
  </div>
{/if}
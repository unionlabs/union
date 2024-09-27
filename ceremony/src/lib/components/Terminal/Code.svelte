<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import Confirm from "$lib/components/Terminal/Confirm.svelte"

const { terminal } = getState()

let inputCode: string = $state("")
let normalizedCode = $derived(normalizeString(inputCode))
let inputElement: HTMLInputElement
let showConfirm = $state(false)
let showInput = $state(true)

function normalizeString(input: string): string {
  return input.toLowerCase().replace(/[^a-z0-9]/gi, "")
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault()
    showConfirm = true
    showInput = false
  }
}

function onCancel() {
  showInput = true
  showConfirm = false
}

onDestroy(() => {
  terminal.clearHistory()
})
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

{#if showConfirm}
  <Confirm normalized={normalizedCode} code={inputCode} {onCancel}/>
{/if}

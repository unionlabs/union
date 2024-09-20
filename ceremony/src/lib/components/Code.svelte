<script lang="ts">
import { callJoinQueue } from "$lib/supabase"
import { toast } from "svelte-sonner"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import Button from "$lib/components/Button.svelte"

type Props = {
  contributor: ContributorState
  secondary?: boolean
}

let { contributor, secondary = false }: Props = $props()

let words: Array<string> = $state(new Array(6).fill(""))
let code = $derived(normalizeString(words))
let codeLoading = $state(false)

function handlePaste(e: ClipboardEvent): void {
  e.preventDefault()
  const pastedText: string = e.clipboardData?.getData("text") || ""
  const pastedWords: Array<string> = pastedText.split(/\s+/).slice(0, 6)
  words = [...pastedWords, ...new Array(6 - pastedWords.length).fill("")]
}

function normalizeString(words: Array<string>): string {
  return words
    .map(word => word.trim().toLowerCase())
    .join("")
    .replace(/[^a-z0-9]/gi, "")
}

async function handleCodeJoin() {
  codeLoading = true
  try {
    console.log(code)
    const codeOk = await callJoinQueue(code)
    if (codeOk) {
      contributor.setAllowanceState("hasRedeemed")
      toast.success("Code successfully redeemed")
    } else {
      toast.error("The code is not valid")
    }
  } catch (error) {
    console.error("Error redeeming code:", error)
    toast.error("An error occurred while redeeming the code")
  } finally {
    codeLoading = false
  }
}

function handleKeyDown(event: KeyboardEvent, index: number) {
  if (event.key === "Enter") {
    event.preventDefault()
    if (index < words.length - 1) {
      // Move to next input
      const nextInput = document.querySelector(
        `input:nth-child(${2 * index + 3})`
      ) as HTMLInputElement
      nextInput?.focus()
    } else {
      // On last input, trigger the USE CODE button
      handleCodeJoin()
    }
  } else if (event.key === "Backspace" && words[index] === "" && index > 0) {
    event.preventDefault()
    // Move to previous input
    const prevInput = document.querySelector(
      `input:nth-child(${2 * index - 1})`
    ) as HTMLInputElement
    prevInput?.focus()
  }
}
</script>

<div class="flex gap-2 max-w-4xl flex-wrap justify-center mb-8">
  {#each words as word, index}
    <input
            bind:value={words[index]}
            onpaste={handlePaste}
            onkeydown={(e) => handleKeyDown(e, index)}
            class="bg-transparent border-b border-white w-20 text-center text-union-accent-500 outline-none focus:ring-0 focus:border-union-accent-500"
            style="--tw-ring-color: transparent;"
    />
    {#if index !== words.length - 1}
      <div class="text-union-accent-500"><p>-</p></div>
    {/if}
  {/each}
</div>

{#if secondary}
  <Button class="bg-transparent text-white hover:text-white border-2 border-white hover:bg-neutral-800" loading={codeLoading} type="button" onclick={handleCodeJoin}>
  Redeem code
  </Button>
{:else}
  <Button loading={codeLoading} type="button" onclick={handleCodeJoin}>
    Redeem code
  </Button>
{/if}

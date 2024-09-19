<script lang="ts">
import { callJoinQueue } from "$lib/supabase"
import { toast } from "svelte-sonner"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import Button from "$lib/components/Button.svelte"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

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
    words = new Array(6).fill("")
  }
}
</script>


<div class="flex gap-2 max-w-4xl flex-wrap justify-center mb-8">
  {#each words as word, index}
    <input
            bind:value={words[index]}
            onpaste={handlePaste}
            class="bg-transparent border-b border-white w-20 text-center text-union-accent-500 outline-none focus:ring-0 focus:border-union-accent-500"
            style="--tw-ring-color: transparent;"
    />
    {#if index !== words.length - 1}
      <div class="text-union-accent-500"><p>-</p></div>
    {/if}
  {/each}
</div>
<Button loading={codeLoading} type="button" onclick={handleCodeJoin}>
  USE CODE
</Button>
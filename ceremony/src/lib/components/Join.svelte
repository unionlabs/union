<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { callJoinQueue, checkIfOpen } from "$lib/supabase"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { toast } from "svelte-sonner"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

let isOpenToPublic = $state(false)
let codeLoading = $state(false)
let waitlistLoading = $state(false)

let words: Array<string> = $state(new Array(6).fill(""))
let code = $derived(normalizeString(words))

function handlePaste(e: ClipboardEvent): void {
  e.preventDefault()
  const pastedText: string = e.clipboardData?.getData("text") || ""
  const pastedWords: Array<string> = pastedText.split(/\s+/).slice(0, 6)
  words = [...pastedWords, ...new Array(6 - pastedWords.length).fill("")]
}

function normalizeString(words: Array<string>): string {
  return words.map(word => word.trim().toLowerCase()).join("")
}

async function handleCodeJoin() {
  codeLoading = true
  try {
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

async function handleWaitlistJoin() {
  waitlistLoading = true
  try {
    await callJoinQueue(null)
    if (isOpenToPublic) {
      contributor.setAllowanceState("inQueue")
      toast.success("Successfully joined the queue")
    } else {
      contributor.setAllowanceState("inWaitlist")
      toast.success("Successfully joined the waitlist")
    }
  } catch (error) {
    console.error("Error joining waitlist:", error)
    toast.error("An error occurred while joining the waitlist")
  } finally {
    waitlistLoading = false
  }
}

const checkOpen = async () => {
  isOpenToPublic = await checkIfOpen()
}

$effect(() => {
  checkOpen()
})
</script>

<div class="text-center flex flex-col gap-4 items-center">
  <H1>Join the ceremony</H1>

  <form class="flex flex-col items-center">
    <div class="flex gap-2 max-w-4xl flex-wrap justify-center mb-4">
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
  </form>

  <div class="flex items-center gap-4">
    <Button loading={codeLoading} type="button" onclick={handleCodeJoin}>
      USE CODE
    </Button>
    <Text>Or</Text>
    <Button loading={waitlistLoading} onclick={handleWaitlistJoin} type="button">
      {isOpenToPublic ? "Join the queue" : "Join the waitlist"}
    </Button>
  </div>
</div>

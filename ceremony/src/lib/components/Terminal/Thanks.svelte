<script lang="ts">
import { getPublicHash } from "$lib/supabase"
import { getState } from "$lib/state/index.svelte.ts"
import { cn, sleep } from "$lib/utils/utils.ts"
import { onDestroy, onMount } from "svelte"
import Button from "$lib/components/Terminal/Button.svelte"
import { beforeNavigate } from "$app/navigation"

const { terminal } = getState()

let focusedIndex = $state(0)
let showButtons = $state(true)

const buttons = $state([
  { text: "Tweet your attestation", action: "tweet" },
  { text: "View contributions", action: "view" }
])

beforeNavigate(() => {
  if (unsubscribe) {
    unsubscribe()
  }
})

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined

onMount(() => {
  terminal.updateHistory("Thank you!", { replace: true })
  terminal.updateHistory("-------------")
  terminal.updateHistory(
    "Your contribution is complete. Thank you for securing the Union network. Tweet your cryptographic attestation for extra transparency.",
    { replace: true }
  )

  subscriptionTimeout = setTimeout(() => {
    unsubscribe = terminal.keys.subscribe(event => {
      if (event) {
        if (event.type === "keydown" && terminal.tab === 1) {
          if (event.key === "ArrowUp") {
            focusedIndex = (focusedIndex - 1 + buttons.length) % buttons.length
          } else if (event.key === "ArrowDown") {
            focusedIndex = (focusedIndex + 1) % buttons.length
          } else if (event.key === "Enter") {
            triggerAction(focusedIndex)
          }
        }
      }
    })
  }, 200)
  return () => {
    if (subscriptionTimeout) {
      clearTimeout(subscriptionTimeout)
    }
    if (unsubscribe) {
      unsubscribe()
    }
  }
})

async function shareOnTwitter() {
  showButtons = false
  terminal.updateHistory("Preparing tweet...", { duplicate: true })
  let hash = await getPublicHash()
  await sleep(2000)
  terminal.updateHistory("Opening X (twitter)...", { duplicate: true })
  await sleep(2000)
  let url = `https://ceremony.union.build/contributions/${hash}`
  const tweetText = `I just contributed to the @union_build Trusted Setup Ceremony, to secure its ZK circuit for trustless, decentralized interoperability. \n\nI attest to my contribution. My public key hash is: \n\n${url}\n\n#JoinTheUnion`
  const twitterIntentUrl = new URL("https://twitter.com/intent/tweet")
  twitterIntentUrl.searchParams.append("text", tweetText)
  window.open(twitterIntentUrl.toString(), "_blank")
  showButtons = true
}

function triggerAction(index: number) {
  if (buttons[index].action === "tweet") {
    shareOnTwitter()
  } else if (buttons[index].action === "view") {
    terminal.setTab(3)
  }
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if showButtons}
  {#each buttons as btn, index}
    <Button
            onmouseenter={() => focusedIndex = index}
            class={cn(index === focusedIndex ? "bg-union-accent-500 text-black" : "")}
            onclick={() => triggerAction(index)}
    >
      &gt; {btn.text}
    </Button>
  {/each}
{/if}
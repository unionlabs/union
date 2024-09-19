<script lang="ts">
import Button from "$lib/components/Button.svelte"
import { getPublicHash } from "$lib/supabase"

let hash = $state(undefined)

async function getHash() {
  hash = await getPublicHash()
}

$effect(() => {
  getHash()
})

function shareOnTwitter() {
  let url = `https://ceremony.union.build/contributions/${hash}`
  const tweetText = `I've contributed to Union Ceremony\n\n${url}`
  const twitterIntentUrl = new URL("https://twitter.com/intent/tweet")
  twitterIntentUrl.searchParams.append("text", tweetText)
  window.open(twitterIntentUrl.toString(), "_blank")
}
</script>

<Button onclick={shareOnTwitter}>
  Tweet This
</Button>
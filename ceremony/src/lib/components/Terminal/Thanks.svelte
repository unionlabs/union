<script lang="ts">
import { getPublicHash } from "$lib/supabase"
import { getState } from "$lib/state/index.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import { onDestroy, onMount } from "svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"

const { terminal } = getState()

onMount(() => {
  terminal.updateHistory({ text: "Thank you!", replace: true })
  terminal.updateHistory({ text: "-------------" })
  terminal.updateHistory({
    text: "Your contribution is complete. Thank you for securing the Union network. Tweet your cryptographic attestation for extra transparency.",
    replace: true
  })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_thanks" }])
})

async function shareOnTwitter() {
  terminal.updateHistory({ text: "Preparing tweet...", duplicate: true })
  let hash = await getPublicHash()
  await sleep(2000)
  terminal.updateHistory({ text: "Opening X (twitter)...", duplicate: true })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "tweet" }])
  await sleep(2000)
  let url = `https://ceremony.union.build/contributions/${hash}`
  const tweetText = `I just contributed to the @union_build Trusted Setup Ceremony, to secure its ZK circuit for trustless, decentralized interoperability. \n\nI attest to my contribution. My public key hash is: \n\n${url}\n\n#JoinTheUnion`
  const twitterIntentUrl = new URL("https://twitter.com/intent/tweet")
  twitterIntentUrl.searchParams.append("text", tweetText)
  window.open(twitterIntentUrl.toString(), "_blank")
}

function trigger(value: "tweet" | "view") {
  if (value === "tweet") {
    shareOnTwitter()
  } else if (value === "view") {
    terminal.setTab(3)
  }
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

<Buttons data={[{text: "Tweet your attestation", action: "tweet"},{text: "View contributions", action: "view"}]} trigger={(value: 'tweet' | 'view') => trigger(value)}/>

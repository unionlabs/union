<script lang="ts">
  import Button from "$lib/components/Button.svelte"
  import {getPublicHash} from "$lib/supabase"

  let hash = $state(undefined)

  async function getHash() {
    hash = await getPublicHash()
  }

  $effect(() => {
    getHash()
  })

  function shareOnTwitter() {
    let url = `https://ceremony.union.build/contributions/${hash}`
    const tweetText = `I just contributed to the @union_build Trusted Setup Ceremony, to secure its ZK circuit for trustless, decentralized interoperability. \n\nI attest to my contribution. My public key hash is: \n\n${url}\n\n#JoinTheUnion`
    const twitterIntentUrl = new URL("https://twitter.com/intent/tweet")
    twitterIntentUrl.searchParams.append("text", tweetText)
    window.open(twitterIntentUrl.toString(), "_blank")
  }
</script>

<button class="block outline-none focus:ring-2 focus:ring-transparent focus:border-none focus:text-union-accent-500"
        onclick={shareOnTwitter}>&gt Tweet your attestation
</button>
<script lang="ts">
import Button from "$lib/components/Button.svelte"

type Props = {
  tweetText: string
  url?: string
}

let { tweetText, url }: Props = $props()

function shareOnTwitter() {
  const twitterIntentUrl = new URL("https://twitter.com/intent/tweet")
  twitterIntentUrl.searchParams.append("text", tweetText)
  if (url) twitterIntentUrl.searchParams.append("url", url)
  window.open(twitterIntentUrl.toString(), "_blank")
}
</script>

<Button onclick={shareOnTwitter}>
  Tweet This
</Button>
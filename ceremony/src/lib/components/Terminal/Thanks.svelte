<script lang="ts">
import { getPublicHash } from "$lib/supabase"
import { getState } from "$lib/state/index.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import { onDestroy, onMount } from "svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import { AddressForm, type ValidState } from "$lib/components/address"

type Actions = "tweet" | "view" | "wallet" | "back"

const { terminal, contributor } = getState()

let showButtons = $state(true)
let showInput = $state(false)

let validation = (val: ValidState) => {
  if (val === "INVALID") {
    showInput = false
    showButtons = true
  } else if (val === "PENDING") {
    showInput = false
  } else if (val === "SAVED") {
    showInput = false
    showButtons = true
  } else if (val === "SKIPPED") {
    showInput = false
  }
}

onMount(() => {
  terminal.setStep(10)
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

function trigger(value: Actions) {
  if (value === "tweet") {
    shareOnTwitter()
  } else if (value === "view") {
    terminal.setTab(3)
  } else if (value === "wallet") {
    showButtons = false
    showInput = true
  } else if (value === "back") {
    showInput = false
    showButtons = true
  }
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if showButtons}
  <Buttons
          data={[{text: "Tweet your attestation", action: "tweet"}, {text: "View contributions", action: "view"}, {text: "Update wallet", action: "wallet"}]}
          trigger={(value: Actions) => trigger(value)}/>
{/if}

{#if showInput}
  <Print>Enter your union or any cosmos address, or type "skip".</Print>
  {#if !contributor.userWallet || contributor.userWallet === "SKIPPED"}
    <Print>No wallet registered</Print>
  {:else }
    <Print>Registered: <span class="text-union-accent-500">{contributor.userWallet}</span></Print>
  {/if}
  <Print><br></Print>
  <div class="flex w-full gap-1">
    <div class="whitespace-nowrap">
      <Print>Enter address:</Print>
    </div>
    <AddressForm {validation}/>
  </div>
{/if}
<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import { generateSecret } from "$lib/client"
import { onDestroy, onMount } from "svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { axiom } from "$lib/utils/axiom.ts"

const { contributor, terminal, user } = getState()

let generated = $state(false)
let generating = $state(false)
let buttons = $state<Array<HTMLButtonElement>>([])
let focusedIndex = $state(0)

onMount(() => {
  terminal.setStep(4)
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_secret" }])
})

function handleDownload() {
  const newUrl = "http://localhost:4919/secret_key"
  window.open(newUrl, "_blank")
}

function stored() {
  localStorage.setItem("downloaded-secret", "true")
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "stored_secret" }])
  contributor.downloadedSecret = true
}

async function generate() {
  if (contributor.state !== "noClient") {
    generating = true
    terminal.updateHistory({ text: "Generating secret..." })
    axiom.ingest("monitor", [{ user: user.session?.user.id, type: "generated_secret" }])
    await sleep(3000)
    generateSecret(user.session?.user.email)
    terminal.updateHistory({ text: "Initialize saving..." })
    await sleep(1000)
    handleDownload()
    generating = false
    generated = true
  }
}

$effect(() => {
  if (generated) {
    terminal.updateHistory({
      text: "Please store your secret somewhere safe, such as in your password manager. There's no need to open the file and remember to never share a secret. This secret key is the only way to prove that you have contributed."
    })
  } else {
    terminal.updateHistory({ text: "Client detected" })
    terminal.updateHistory({ text: "Generate your PGP secret" })
    terminal.updateHistory({
      text: "The MPC client automatically uses this secret to sign your contribution."
    })
    terminal.updateHistory({ text: "Your secret is locally generated through the MPC client." })
  }
})

onDestroy(() => {
  terminal.clearHistory()
})

function trigger(value: "generate" | "stored") {
  if (value === "generate") {
    generate()
  } else if (value === "stored") {
    stored()
  }
}
</script>

{#if !generating}
  {#if !generated}
    <Buttons data={[{text: "Generate secret", action: "generate"}]} trigger={(value) => trigger(value)}/>
  {:else}
    <Buttons
            data={[{text: "Generate secret again", action: "generate"}, {text: "I've stored my secret", action: "stored"}]}
            trigger={(value) => trigger(value)}/>
  {/if}
{/if}

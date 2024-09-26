<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import { generateSecret } from "$lib/client"
import Button from "$lib/components/Terminal/Button.svelte"

const { contributor, terminal, user } = getState()

let generated = $state(false)
let generating = $state(false)

function handleDownload() {
  const newUrl = "http://localhost:4919/secret_key"
  window.open(newUrl, "_blank")
}

function setDownloadedSecret() {
  localStorage.setItem("downloaded-secret", "true")
  contributor.downloadedSecret = true
}

async function generate() {
  if (contributor.state !== "noClient") {
    generating = true
    terminal.updateHistory("Generating secret...")
    await sleep(3000)
    generateSecret(user.session?.user.email)
    terminal.updateHistory("Initialize saving...")
    await sleep(1000)
    handleDownload()
    generating = false
    generated = true
  }
}

$effect(() => {
  if (generated) {
    terminal.updateHistory(
      "Please store your secret somewhere safe, such as in your password manager. There's no need to open the file and remember to never share a secret. This secret key is the only way to prove that you have contributed."
    )
  } else {
    terminal.updateHistory("Client detected")
    terminal.updateHistory("Generate your PGP secret")
    terminal.updateHistory(
      "The MPC client automatically uses this secret to sign your contribution."
    )
    terminal.updateHistory("Your secret is locally generated through the MPC client.")
  }
})
</script>

{#if !generating}
  {#if !generated}
    <Button
            onclick={generate} autofocus>&gt Generate secret
    </Button>
  {:else}
    <Button autofocus onclick={setDownloadedSecret}>&gt I've generated and stored my secret</Button>
    <Button onclick={generate}>&gt Generate again</Button>
  {/if}
{/if}

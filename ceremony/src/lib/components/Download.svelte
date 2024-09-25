<script lang="ts">
import Button from "$lib/components/Button.svelte"
import type {Contributor} from "$lib/state/contributor.svelte.ts";
import Print from "$lib/components/TerminalApp/Print.svelte";

type Props = {
  contributor: Contributor
}

let { contributor }: Props = $props()

let download = $state(false)

function handleDownload(event: MouseEvent) {
  event.preventDefault()
  const newUrl = "http://localhost:4919/secret_key"
  window.open(newUrl, "_blank")
  download = true
}

function setDownloadedSecret() {
  localStorage.setItem("downloaded-secret", "true")
  contributor.downloadedSecret = true
}
</script>

{#if !download}
  <Print>Generate your PGP secret</Print>
  <Print>
    The MPC client automatically uses this secret to sign your contribution.<br>
    Your secret is locally generated through the MPC client.
  </Print>
  <Button variant="primary" onclick={handleDownload}>Generate secret</Button>
{:else}
  <Print>Store your PGP secret</Print>
  <Print>
    Please store your secret somewhere safe, such as in your password manager.
    <br> There's no need to open the file and remember to never share a secret.
    <br> This secret key is the only way to prove that you have contributed.
  </Print>
  <div>
    <Button variant="primary" onclick={setDownloadedSecret}>I've generated and stored my secret</Button>
    <Button variant="secondary" href="http://localhost:4919/secret_key" target="_blank">Generate again</Button>
  </div>
{/if}
<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import SharpWalletIcon from "$lib/components/icons/SharpWalletIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { Option } from "effect"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { chains } from "$lib/stores/chains.svelte.ts"
import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
import { Turnstile } from "svelte-turnstile"
import request from "graphql-request"
import { writable } from "svelte/store"
import { faucetUnoMutationDocument } from "$lib/queries/faucet"
import { URLS } from "$lib/constants"
import { Data } from "effect"
import { bech32, bytes } from "@scure/base"

// Define the faucet state type using Data.TaggedEnum.
type FaucetProcessState = Data.TaggedEnum<{
  Idle: {}
  Verifying: {}
  Verified: { token: string }
  Submitting: { token: string }
  Success: { message: string }
  Failure: { error: string }
}>

// Create the tagged enum instance.
const FaucetProcess = Data.taggedEnum<FaucetProcessState>()

// Initialize the faucet process state to Idle.
const faucetProcess = writable<FaucetProcessState>(FaucetProcess.Idle())

// Variables for managing the Turnstile component.
let resetTurnstile: () => void
let showTurnstile = false

// When the user clicks "Claim", trigger verification.
// For now, we bypass verification and use a dummy token.
const startVerification = () => {
  faucetProcess.set(FaucetProcess.Verifying())
  showTurnstile = true
  resetTurnstile?.() // resets/retriggers the Turnstile if available
}

// Callback for successful Turnstile captcha.
const handleTurnstileCallback = (
  e: CustomEvent<{ token: string; preClearanceObtained: boolean }>
) => {
  const token = e.detail.token
  faucetProcess.set(FaucetProcess.Verified({ token }))
  // Immediately submit the faucet request.
  submitFaucetRequest(token)
}

// Callback for a Turnstile error.
const handleTurnstileError = (e: CustomEvent<{ code: string }>) => {
  faucetProcess.set(FaucetProcess.Failure({ error: `Verification error: ${e.detail.code}` }))
  showTurnstile = false
}

// Submit the faucet request and update the state accordingly.
const submitFaucetRequest = async (token: string) => {
  faucetProcess.set(FaucetProcess.Submitting({ token }))
  try {
    let wallet_addr = ""
    if (Option.isSome(wallets.cosmosAddress)) {
      const words = bech32.toWords(bytes("hex", wallets.cosmosAddress.value.slice(2)))
      wallet_addr = bech32.encode("union", words)
    }
    const result = await request(URLS().GRAPHQL, faucetUnoMutationDocument, {
      chainId: "union-testnet-10",
      denom: "muno",
      address: wallet_addr,
      captchaToken: token
    })
    console.info("faucet result is:", result)
    if (!result || !result.drip_drop || !result.drip_drop.send) {
      faucetProcess.set(FaucetProcess.Failure({ error: "Empty faucet response" }))
      showTurnstile = false
      return
    }

    if (result.drip_drop.send.startsWith("ERROR")) {
      faucetProcess.set(
        FaucetProcess.Failure({ error: `Error from faucet: ${result.drip_drop.send}` })
      )
      showTurnstile = false
      return
    }

    faucetProcess.set(FaucetProcess.Success({ message: result.drip_drop.send }))
    showTurnstile = false
  } catch (error) {
    console.info("error is:", error)
    faucetProcess.set(FaucetProcess.Failure({ error: `Faucet error: ${error}` }))
    showTurnstile = false
  }
}

// Reset the faucet process to allow a new request.
const resetProcess = () => {
  faucetProcess.set(FaucetProcess.Idle())
  showTurnstile = false
}
</script>

<Card divided class="self-center">
  <div class="p-4 flex gap-1 ">
    <h2>UNO Faucet</h2>
  </div>
  {#if Option.isSome(chains.data)}
    {@const unionTestnet10 = Option.fromNullable(chains.data.value.find(c => c.universal_chain_id === "union.union-testnet-10"))}
    <div class="flex flex-col gap-4 p-4">
      <div>
        <p>Official faucet for the UNO testnet token.</p>
        <p>This faucet is protected by CloudFlare Turnstile.</p>
        <p>You can use this faucet once a day.</p>
      </div>
      <div>
        <div class="flex items-center mr-5 text-zinc-400 justify-self-end">
          {#if Option.isSome(wallets.cosmosAddress) && Option.isSome(unionTestnet10)}
            <p class="text-xs mb-2">
              <AddressComponent truncate address={wallets.cosmosAddress.value} chain={unionTestnet10.value}/>
            </p>
          {:else}
            <p class="text-xs mb-2">No receiver</p>
          {/if}
          <AngleArrowIcon class="rotate-270"/>
        </div>
          {#if $faucetProcess._tag === "Idle"}
          <div class="flex gap-4">
            <Button onclick={startVerification} class="flex-1"
            disabled={!Option.isSome(wallets.cosmosAddress)}
                  >Claim</Button>
            <Button><SharpWalletIcon class="size-5"/></Button>
          </div>
        {:else if $faucetProcess._tag === "Verifying"}
          <div class="flex flex-col items-center">
            <p class="text-xs">Verifying, please complete captcha...</p>
            {#if showTurnstile}
              <Turnstile
                siteKey="0x4AAAAAAA-eVs5k0b8Q1dl5"
                on:callback={handleTurnstileCallback}
                on:error={handleTurnstileError}
                theme="auto"
                size="normal"
                bind:reset={resetTurnstile}
              />
            {/if}
          </div>
        {:else if $faucetProcess._tag === "Submitting"}
          <div class="flex flex-col items-center">
            <p class="text-xs">Submitting faucet request...</p>
          </div>
        {:else if $faucetProcess._tag === "Success"}
          <div class="flex flex-col items-center">
            <p class="text-xs">Tokens sent! Transaction hash:</p>
            <p class="text-xs break-all">
              <a href={`https://explorer.testnet-10.union.build/union/tx/${$faucetProcess.message}`} target="_blank">
                {$faucetProcess.message}
              </a>
            </p>
            <Button onclick={resetProcess} class="mt-2">New Request</Button>
          </div>
        {:else if $faucetProcess._tag === "Failure"}
          <div class="flex flex-col items-center">
            <p class="text-xs text-red-500">Error: {$faucetProcess.error}</p>
            <Button onclick={resetProcess} class="mt-2">Retry</Button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</Card>

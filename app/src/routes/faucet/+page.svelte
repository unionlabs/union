<script lang="ts">
import request from "graphql-request"
import { onDestroy, onMount } from "svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import { URLS } from "$lib/constants/index.ts"
import { Label } from "$lib/components/ui/label"
import { writable, type Writable } from "svelte/store"
import Truncate from "$lib/components/truncate.svelte"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import SpinnerSVG from "$lib/components/spinner-svg.svelte"
import WalletGateCosmos from "$lib/components/wallet-gate-cosmos.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import type { DiscriminatedUnion } from "$lib/utilities/types.ts"
import { faucetUnoMutation2 } from "$lib/graphql/queries/faucet.ts"
import { isValidCosmosAddress } from "$lib/wallet/utilities/validate.ts"
import { createCosmosSdkAddressRegex } from "$lib/utilities/address.ts"
import { Turnstile } from "svelte-turnstile"

// import ExternalFaucets from "./(components)/external-faucets.svelte"
// import DydxFaucet from "./(components)/dydx-faucet.svelte"
// import StrideFaucet from "./(components)/stride-faucet.svelte"

type FaucetState = DiscriminatedUnion<
  "kind",
  {
    IDLE: {}
    VERIFYING: {}
    VERIFIED: {}
    SUBMITTING: { captchaToken: string }
    RESULT_OK: { message: string }
    RESULT_ERR: { error: string }
    VERIFICATION_FAILED: { error: string }
  }
>

let address = ""
let turnstileToken = ""
let resetTurnstile: () => void
let showTurnstile = false

onMount(() => {
  address = $cosmosStore.address ?? ""
})

const unsubscribe = cosmosStore.subscribe(v => {
  if (address !== v.address) {
    address = v.address ?? ""
  }
})

onDestroy(() => {
  unsubscribe()
})

const resetInput = () => {
  address = $cosmosStore.address ?? ""
}

let unoFaucetState: Writable<FaucetState> = writable({ kind: "IDLE" })

const verifyWithTurnstile = () => {
  if ($unoFaucetState.kind === "IDLE") {
    showTurnstile = true
    unoFaucetState.set({ kind: "VERIFYING" })
    resetTurnstile?.() // Trigger verification
  }
}

const requestUnoFromFaucet = async () => {
  if ($unoFaucetState.kind === "VERIFIED") {
    unoFaucetState.set({ kind: "SUBMITTING", captchaToken: turnstileToken })
  }

  if ($unoFaucetState.kind === "SUBMITTING") {
    try {
      const result = await request(URLS().GRAPHQL, faucetUnoMutation2, {
        chainId: "union-testnet-9",
        denom: "muno",
        address,
        captchaToken: $unoFaucetState.captchaToken
      })
      if (result.send === null) {
        unoFaucetState.set({
          kind: "RESULT_ERR",
          error: "Empty faucet response"
        })
        turnstileToken = ""
        showTurnstile = false
        return
      }

      if (result.send.startsWith("ERROR")) {
        console.error(result.send)
        unoFaucetState.set({ kind: "RESULT_ERR", error: `Error from faucet` })
        turnstileToken = ""
        showTurnstile = false
        return
      }

      unoFaucetState.set({
        kind: "RESULT_OK",
        message: result.send
      })
      turnstileToken = ""
      showTurnstile = false
    } catch (error) {
      unoFaucetState.set({
        kind: "RESULT_ERR",
        error: `Faucet error: ${error}`
      })
      turnstileToken = ""
      showTurnstile = false
      return
    }
  }
}

const resetVerification = () => {
  if ($unoFaucetState.kind === "VERIFICATION_FAILED") {
    turnstileToken = ""
    showTurnstile = false
    unoFaucetState.set({ kind: "IDLE" })
  }
}

const handleTurnstileCallback = (
  e: CustomEvent<{ token: string; preClearanceObtained: boolean }>
) => {
  turnstileToken = e.detail.token
  if ($unoFaucetState.kind === "VERIFYING") {
    unoFaucetState.set({ kind: "VERIFIED" })
  }
}

const handleTurnstileError = (e: CustomEvent<{ code: string }>) => {
  if ($unoFaucetState.kind === "VERIFYING") {
    unoFaucetState.set({
      kind: "VERIFICATION_FAILED",
      error: `Verification error: ${e.detail.code}`
    })
  }
}
</script>

<svelte:head>
  <title>Union | Faucet</title>
</svelte:head>

<main class="flex flex-col gap-6 items-center overflow-y-scroll max-h-full py-6 px-3 sm:px-6 w-full dark:bg-muted">
  <Card.Root class="w-full max-w-lg">
    <Card.Header>
      <Card.Title>UNO Drip Faucet</Card.Title>
      <Card.Description>
        Official faucet for Union's native gas token.
      </Card.Description>
    </Card.Header>
    <Card.Content>
      {#if $unoFaucetState.kind === "RESULT_OK"}
        <p>
          Tokens sent: <a
                href={`https://explorer.testnet-9.union.build/union/tx/${$unoFaucetState.message}`}
        >
          <Truncate
                  class="underline"
                  value={$unoFaucetState.message}
                  type="hash"
          />
        </a>
        </p>
      {:else if $unoFaucetState.kind === "RESULT_ERR"}
        <p class="mb-4">
          Sorry, there was an error while using the faucet. Did you make sure
          that the address is correct?
        </p>
        <Button on:click={() => unoFaucetState.set({ kind: "IDLE" })}>
          Retry
        </Button>
        <p class="mt-4 break-words text-xs">{$unoFaucetState.error}</p>
      {:else}
        <form
                action="?"
                method="POST"
                class="flex flex-col w-full gap-4"
                name="faucet-form"
                on:submit|preventDefault
        >
          <div>
            <Label for="address">Address</Label>
            <div class="flex items-start gap-2">
              <div class="w-full">
                <div class="relative w-full mb-2">
                  <Input
                          autocapitalize="none"
                          autocomplete="off"
                          autocorrect="off"
                          bind:value={address}
                          id="address"
                          pattern={createCosmosSdkAddressRegex({ prefix: "union" }).source}
                          placeholder="union14ea6..."
                          required={true}
                          minlength={44}
                          maxlength={44}
                          spellcheck="false"
                          name="wallet-address"
                          type="text"
                          data-1p-ignore={true}
                          data-lpignore={true}
                          class="disabled:opacity-100 disabled:bg-black/20"
                  />
                </div>
                <div class="flex justify-between px-1">
                  <div class="text-xs">
                    <ChainsGate let:chains>
                      <WalletGateCosmos>
                        <!--
                        <p slot="connected" let:userAddrCosmos>
                          <span class="text-muted-foreground">Balance: </span>
                            <TokenBalance
                              {chains}
                              {userAddrCosmos}
                              symbol="muno"
                            />
                        </p>
                        !-->
                        <p slot="disconnected">
                          Connect cosmos wallet
                        </p>
                      </WalletGateCosmos>
                    </ChainsGate>
                  </div>
                  {#if address !== $cosmosStore.address}
                    <button
                            type="button"
                            on:click={resetInput}
                            class="text-xs text-muted-foreground hover:text-primary transition"
                    >
                      Reset
                    </button>
                  {/if}
                </div>
              </div>
            </div>
          </div>
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
          <div class="flex flex-row items-center gap-4">
            {#if $unoFaucetState.kind === "IDLE" || $unoFaucetState.kind === "VERIFYING"}
              <Button
                      type="button"
                      on:click={(event) => {
                  event.preventDefault()
                  verifyWithTurnstile()
                }}
                      disabled={!isValidCosmosAddress(address, ["union"]) ||
                  $unoFaucetState.kind === "VERIFYING"}
                      class={cn("min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50")}
              >
                Verify
                {#if $unoFaucetState.kind === "VERIFYING"}
                  <span class="ml-2">
                    <SpinnerSVG className="w-4 h-4" />
                  </span>
                {/if}
              </Button>
            {:else if $unoFaucetState.kind === "VERIFIED" || $unoFaucetState.kind === "SUBMITTING"}
              <Button
                      type="button"
                      on:click={(event) => {
                  event.preventDefault()
                  requestUnoFromFaucet()
                }}
                      disabled={$unoFaucetState.kind === "SUBMITTING"}
                      class={cn("min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50")}
              >
                Submit
                {#if $unoFaucetState.kind === "SUBMITTING"}
                  <span class="ml-2">
                    <SpinnerSVG className="w-4 h-4" />
                  </span>
                {/if}
              </Button>
            {:else if $unoFaucetState.kind === "VERIFICATION_FAILED"}
              <Button
                      type="button"
                      on:click={(event) => {
                  event.preventDefault()
                  resetVerification()
                }}
                      class={cn("min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50")}
              >
                Reset
              </Button>
              <p class="text-xs text-red-500">{$unoFaucetState.error}</p>
            {/if}
            <div class="text-[10px]">
              This faucet is protected by Cloudflare Turnstile.
            </div>
          </div>
        </form>
      {/if}
    </Card.Content>
  </Card.Root>

<!--  <DydxFaucet />-->
<!--  <StrideFaucet />-->
  <!--
  <ChainsGate let:chains>
    <ExternalFaucets {chains} />
  </ChainsGate>
  !-->
</main>
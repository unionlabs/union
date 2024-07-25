<script lang="ts">
import request from "graphql-request"
import { cn } from "$lib/utilities/shadcn.ts"
import { URLS } from "$lib/constants/index.ts"
import { Label } from "$lib/components/ui/label"
import { writable, type Writable } from "svelte/store"
import Truncate from "$lib/components/truncate.svelte"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import DydxFaucet from "./(components)/dydx-faucet.svelte"
import { Button } from "$lib/components/ui/button/index.ts"
import SpinnerSVG from "$lib/components/spinner-svg.svelte"
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import TokenBalance from "./(components)/token-balance.svelte"
import ExternalFaucets from "./(components)/external-faucets.svelte"
import { faucetUnoMutation2 } from "$lib/graphql/documents/faucet.ts"
import { isValidCosmosAddress } from "$lib/wallet/utilities/validate.ts"
import { cosmosChainAddressTransfers } from "$lib/queries/transfers/cosmos"
import type { AwaitedReturnType, DiscriminatedUnion } from "$lib/utilities/types.ts"
import { convertCosmosAddress, createCosmosSdkAddressRegex } from "$lib/utilities/address.ts"
import { onDestroy, onMount } from "svelte"

type FaucetState = DiscriminatedUnion<
  "kind",
  {
    IDLE: {}
    REQUESTING_TOKEN: {}
    SUBMITTING: { captchaToken: string }
    RESULT_OK: { message: string }
    RESULT_ERR: { error: string }
  }
>

let address = ""

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

const requestUnoFromFaucet = async () => {
  if ($unoFaucetState.kind === "IDLE" || $unoFaucetState.kind === "REQUESTING_TOKEN") {
    unoFaucetState.set({ kind: "REQUESTING_TOKEN" })

    if (!window?.__google_recaptcha_client) {
      console.error("Recaptcha client not loaded")
      unoFaucetState.set({
        kind: "RESULT_ERR",
        error: "Recaptcha client not loaded"
      })
      return
    }

    if (
      typeof window.grecaptcha === "undefined" ||
      typeof window.grecaptcha.execute !== "function"
    ) {
      console.error("Recaptcha execute function not available")
      unoFaucetState.set({
        kind: "RESULT_ERR",
        error: "Recaptcha execute function not available"
      })
      return
    }

    const captchaToken = await window.grecaptcha.execute(
      "6LdaIQIqAAAAANckEOOTQCFun1buOvgGX8J8ocow",
      { action: "submit" }
    )

    unoFaucetState.set({ kind: "SUBMITTING", captchaToken })
  }

  if ($unoFaucetState.kind === "SUBMITTING") {
    try {
      const result = await request(URLS.GRAPHQL, faucetUnoMutation2, {
        address,
        captchaToken: $unoFaucetState.captchaToken
      })
      if (result.faucet2 === null) {
        unoFaucetState.set({
          kind: "RESULT_ERR",
          error: "Empty faucet response"
        })
        return
      }

      if (result.faucet2.send.startsWith("ERROR")) {
        console.error(result.faucet2.send)
        unoFaucetState.set({ kind: "RESULT_ERR", error: `Error from faucet` })
        return
      }

      unoFaucetState.set({
        kind: "RESULT_OK",
        message: result.faucet2.send
      })
    } catch (error) {
      unoFaucetState.set({
        kind: "RESULT_ERR",
        error: `Faucet error: ${error}`
      })
      return
    }
  }
}
</script>

<svelte:head>
  <title>Union | Faucet</title>
  <script
    src="https://www.google.com/recaptcha/api.js?render=6LdaIQIqAAAAANckEOOTQCFun1buOvgGX8J8ocow"
    async
    defer
  ></script>
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
            href={`https://explorer.testnet-8.union.build/union/tx/${$unoFaucetState.message}`}
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
          on:submit|preventDefault|once={requestUnoFromFaucet}
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
                    pattern={createCosmosSdkAddressRegex({ prefix: "union" })
                      .source}
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
                      <WalletGate
                        let:userAddr
                        let:connected
                        let:cosmosConnected
                      >
                        <p>
                          <span class="text-muted-foreground">Balance: </span>
                          {#if cosmosConnected}
                            <TokenBalance
                              {chains}
                              {userAddr}
                              {connected}
                              symbol="muno"
                            />
                          {:else}
                            Connect cosmos wallet
                          {/if}
                        </p>
                      </WalletGate>
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
          <div class="flex flex-row items-center gap-4">
            <Button
              type="submit"
              on:click={(event) => {
                event.preventDefault()
                requestUnoFromFaucet()
              }}
              disabled={$unoFaucetState.kind !== "IDLE" ||
                isValidCosmosAddress(address, ["union"]) === false}
              class={cn(
                "min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50"
              )}
            >
              Submit
              {#if $unoFaucetState.kind !== "IDLE"}
                <span class="ml-2">
                  <SpinnerSVG className="w-4 h-4" />
                </span>
              {/if}
            </Button>
            <div class="text-[10px]">
              This faucet is protected by reCAPTCHA and the Google <a
                class="underline"
                href="https://policies.google.com/privacy"
              >
                Privacy Policy
              </a>
              and
              <a class="underline" href="https://policies.google.com/terms">
                Terms of Service
              </a> apply.
            </div>
          </div>
          <div
            class="g-recaptcha sr-only"
            data-sitekey="6LdaIQIqAAAAANckEOOTQCFun1buOvgGX8J8ocow"
            data-callback="onSubmit"
            data-size="invisible"
          ></div>
        </form>
      {/if}
    </Card.Content>
  </Card.Root>
  <!-- dydx faucet -->
  <DydxFaucet />
  <ChainsGate let:chains>
    <ExternalFaucets {chains} />
  </ChainsGate>
</main>

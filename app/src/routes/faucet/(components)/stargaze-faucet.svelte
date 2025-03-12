<script lang="ts">
import { URLS } from "$lib/constants"
import request from "graphql-request"
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import { createQuery } from "@tanstack/svelte-query"
import Truncate from "$lib/components/truncate.svelte"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import SpinnerSVG from "$lib/components/spinner-svg.svelte"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { derived, writable, type Writable } from "svelte/store"
import { getCosmosChainBalances } from "$lib/queries/balance/cosmos"
import { createCosmosSdkAddressRegex } from "$lib/utilities/address.ts"
import { bech32ToBech32Address, isValidBech32Address } from "@unionlabs/client"
import type { AwaitedReturnType, DiscriminatedUnion } from "$lib/utilities/types.ts"
import { faucetUnoMutation2 } from "$lib/graphql/queries/faucet.ts"
import { Turnstile } from "svelte-turnstile"

type DydxFaucetState = DiscriminatedUnion<
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

let stargazeAddress = derived(cosmosStore, $cosmosStore =>
  $cosmosStore.address
    ? bech32ToBech32Address({
        address: $cosmosStore.address,
        toPrefix: "stars"
      })
    : ""
)

let stargazeFaucetState: Writable<DydxFaucetState> = writable({
  kind: "IDLE"
})
let turnstileToken = ""
let resetTurnstile: () => void
let showTurnstile = false

const verifyWithTurnstile = () => {
  if ($stargazeFaucetState.kind === "IDLE") {
    showTurnstile = true
    stargazeFaucetState.set({ kind: "VERIFYING" })
    resetTurnstile?.()
  }
}

const requestStarsFromFaucet = async () => {
  console.info("stargazeAddress: ", $stargazeAddress)

  if ($stargazeFaucetState.kind === "VERIFIED") {
    stargazeFaucetState.set({
      kind: "SUBMITTING",
      captchaToken: turnstileToken
    })
  }

  if ($stargazeFaucetState.kind === "SUBMITTING") {
    try {
      const result = await request(URLS().GRAPHQL, faucetUnoMutation2, {
        chainId: "elgafar-1",
        denom: "ustars",
        address: $stargazeAddress,
        captchaToken: $stargazeFaucetState.captchaToken
      })

      if (!result.send) {
        stargazeFaucetState.set({
          kind: "RESULT_ERR",
          error: "Empty faucet response"
        })
        turnstileToken = ""
        showTurnstile = false
        return
      }

      if (result.send.startsWith("ERROR")) {
        console.error(result.send)
        stargazeFaucetState.set({
          kind: "RESULT_ERR",
          error: result.send.endsWith("ratelimited")
            ? "You already got USTARS from the faucet today. Try again in 24 hours."
            : "Error from faucet"
        })
        turnstileToken = ""
        showTurnstile = false
        return
      }

      stargazeFaucetState.set({
        kind: "RESULT_OK",
        message: result.send
      })
      turnstileToken = ""
      showTurnstile = false
    } catch (error) {
      console.error(error)
      stargazeFaucetState.set({
        kind: "RESULT_ERR",
        error: `Faucet error: ${error}`
      })
      turnstileToken = ""
      showTurnstile = false
    }
  }
}

const resetVerification = () => {
  if ($stargazeFaucetState.kind === "VERIFICATION_FAILED") {
    turnstileToken = ""
    showTurnstile = false
    stargazeFaucetState.set({ kind: "IDLE" })
  }
}

const handleTurnstileCallback = (
  e: CustomEvent<{ token: string; preClearanceObtained: boolean }>
) => {
  turnstileToken = e.detail.token
  if ($stargazeFaucetState.kind === "VERIFYING") {
    stargazeFaucetState.set({ kind: "VERIFIED" })
  }
}

const handleTurnstileError = (e: CustomEvent<{ code: string }>) => {
  if ($stargazeFaucetState.kind === "VERIFYING") {
    stargazeFaucetState.set({
      kind: "VERIFICATION_FAILED",
      error: `Verification error: ${e.detail.code}`
    })
  }
}

let stargazeBalance = createQuery(
  derived(stargazeAddress, $stargazeAddress => ({
    queryKey: ["stargaze-balance", $stargazeAddress],
    enabled: $stargazeAddress?.indexOf("stars") === 0,
    refetchInterval: () => ($stargazeAddress?.indexOf("stars") === 0 ? 5_000 : false),
    queryFn: async () =>
      await getCosmosChainBalances({
        walletAddress: `${$stargazeAddress}`,
        url: "https://stargaze-testnet-api.polkachu.com"
      }),
    select: (data: AwaitedReturnType<typeof getCosmosChainBalances>) =>
      data?.find(balance => balance?.symbol === "ustars")
  }))
)
</script>

<!-- stargaze faucet -->
<Card.Root
  class={cn(
    "w-full max-w-lg rounded-lg font-sans",
    "bg-[#181825] text-[#99f0cf] dark:bg-[#2D2D44]/50 dark:text-[#99f0cf]",
  )}
>
  <Card.Header>
    <Card.Title class="flex justify-between select-none">
      <p class="flex gap-x-3">
        <a
          target="_blank"
          rel="noopener noreferrer"
          href="https://www.stargaze.zone/"
        >
          <img src="/images/logo/stargaze-logo.svg" alt="" class="w-10" />
        </a>
        Stargaze Faucet
      </p>
    </Card.Title>
  </Card.Header>
  <Card.Content>
    {#if $stargazeFaucetState.kind === "RESULT_OK"}
      <p>
        Tokens sent: <a
          target="_blank"
          rel="noopener noreferrer"
          href={`https://testnet.ping.pub/stargaze/tx/${$stargazeFaucetState.message}`}
        >
          <Truncate
            class="underline"
            value={$stargazeFaucetState.message}
            type="hash"
          />
        </a>
      </p>
    {:else if $stargazeFaucetState.kind === "RESULT_ERR"}
      <p class="mb-4">
        {$stargazeFaucetState.error}
      </p>
      <Button
        class={cn(
          "bg-[#99f0cf] text-[#010001] dark:bg-[#99f0cf] dark:text-[#010001]",
          "disabled:opacity-100 disabled:bg-black/20 rounded-md focus:ring-0 focus-visible:ring-0",
        )}
        on:click={() => stargazeFaucetState.set({ kind: "IDLE" })}
      >
        Retry
      </Button>
    {:else}
      <form
        action="?"
        method="POST"
        name="faucet-form"
        class="flex flex-col w-full gap-4"
        on:submit|preventDefault
      >
        <div>
          <Label for="address">Address</Label>
          <div class="flex items-start gap-2">
            <div class="w-full">
              <div class="relative w-full mb-2">
                <Input
                  type="text"
                  minlength={44}
                  maxlength={44}
                  readonly={true}
                  required={true}
                  autocorrect="off"
                  id="stargaze-address"
                  autocomplete="off"
                  spellcheck="false"
                  autocapitalize="none"
                  value={$stargazeAddress}
                  data-lpignore={true}
                  data-1p-ignore={true}
                  placeholder="stars14ea6…"
                  name="stargaze-wallet-address"
                  class={cn(
                    "bg-[#2D2D44] text-[#ffffff] dark:bg-[#181825] dark:text-[#ffffff]",
                    "disabled:opacity-100 disabled:bg-black/20 rounded-md focus:ring-0 focus-visible:ring-0",
                  )}
                  pattern={createCosmosSdkAddressRegex({ prefix: "stargaze" })
                    .source}
                />
              </div>
              <div class="flex justify-between px-1">
                <div class="text-xs">
                  <p>
                    {#if $stargazeAddress?.indexOf("stars") === 0 && $stargazeBalance.status === "success"}
                      <!--
                      <span>Balance: </span>
                      {$stargazeBalance?.data?.balance ?? 0}
                      ustars
                      !-->
                    {:else}
                      Connect cosmos wallet
                    {/if}
                  </p>
                </div>
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
          {#if $stargazeFaucetState.kind === "IDLE" || $stargazeFaucetState.kind === "VERIFYING"}
            <Button
              type="button"
              on:click={(event) => {
                event.preventDefault();
                verifyWithTurnstile();
              }}
              disabled={!isValidBech32Address($stargazeAddress) ||
                $stargazeFaucetState.kind === "VERIFYING"}
              class={cn(
                "min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50 rounded-md",
                "bg-[#99f0cf] text-[#010001] dark:bg-[#99f0cf] dark:text-[#010001]",
              )}
            >
              Verify
              {#if $stargazeFaucetState.kind === "VERIFYING"}
                <span class="ml-2">
                  <SpinnerSVG className="w-4 h-4" />
                </span>
              {/if}
            </Button>
          {:else if $stargazeFaucetState.kind === "VERIFIED" || $stargazeFaucetState.kind === "SUBMITTING" || true}
            <Button
              type="button"
              on:click={(event) => {
                event.preventDefault();
                requestStarsFromFaucet();
              }}
              disabled={$stargazeFaucetState.kind === "SUBMITTING"}
              class={cn(
                "min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50 rounded-md",
                "bg-[#99f0cf] text-[#010001] dark:bg-[#99f0cf] dark:text-[#010001]",
              )}
            >
              Submit
              {#if $stargazeFaucetState.kind === "SUBMITTING"}
                <span class="ml-2">
                  <SpinnerSVG className="w-4 h-4" />
                </span>
              {/if}
            </Button>
          {:else if $stargazeFaucetState.kind === "VERIFICATION_FAILED"}
            <Button
              type="button"
              on:click={(event) => {
                event.preventDefault();
                resetVerification();
              }}
              class={cn(
                "min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50 rounded-md",
                "bg-[#99f0cf] text-[#ffffff] dark:bg-[#99f0cf] dark:text-[#ffffff]",
              )}
            >
              Reset
            </Button>
            <p class="text-xs text-red-500">{$stargazeFaucetState.error}</p>
          {/if}
              <p class="text-[10px]">
                This faucet is protected by Cloudflare Turnstile. Funds provided by 
  <a
                class="text-[#13ffa4]"
                target="_blank"
                rel="noopener noreferrer"
                href="https://www.stargaze.zone/"
        >
        stargaze.zone.
        </a>
          </p>
        </div>
      </form>
    {/if}
  </Card.Content>
</Card.Root>

<script lang="ts">
  import toast from "svelte-french-toast"
  import { debounce } from "$lib/utilities/index.ts"
  import LockLockedIcon from "virtual:icons/lucide/lock"
  import { Input } from "$lib/components/ui/input/index.ts"
  import LockOpenIcon from "virtual:icons/lucide/lock-open"
  import { Button } from "$lib/components/ui/button/index.ts"
  import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
  import LoadingIcon from "virtual:icons/lucide/loader-circle"
  import { unionTransfersQuery } from "$lib/queries/transfers.ts"
  import ExternalLinkIcon from "virtual:icons/lucide/external-link"
  import { unionAddressRegex } from "./schema.ts"
  import { isValidCosmosAddress } from "$/lib/wallet/utilities/validate.ts"
  import { Label } from "$lib/components/ui/label";
  import { getUnoFromFaucet } from "$lib/mutations/faucet.ts";
  import { createMutation, createQuery } from "@tanstack/svelte-query";
  import * as Form from "$lib/components/ui/form/index.ts"
  import * as Card from "$lib/components/ui/card/index.ts"

  interface Balance {
    amount: string;
    denom: string;
  }

  let userInput = false;
  let address: string = $cosmosStore.address ?? '';

  $: if (!userInput && ($cosmosStore.address !== address)) {
    address = $cosmosStore.address ?? '';
  }

  const handleInput = (event: Event) => {
    address = (event.target as HTMLInputElement).value;
    userInput = true;
  }

  const resetInput = () => {
    userInput = false;
    address = $cosmosStore.address ?? '';
  }

  const handleInputCommit = () => {
    userInput = false;
  }

  const debounceDelay = 3_500
  let submissionStatus: "idle" | "submitting" | "submitted" | "error" = "idle"
  let inputState: "locked" | "unlocked" = $cosmosStore.address ? "locked" : "unlocked"
  const onLockClick = () => (inputState = inputState === "locked" ? "unlocked" : "locked")

  $: {
    if (submissionStatus === "submitting") {
      toast.loading("Submitting faucet request 🚰", {
        duration: debounceDelay - 300,
        className: "text-sm p-2.5"
      })
    }
  }

  let opacity = 0
  let focused = false
  let input: HTMLInputElement
  let position = { x: 0, y: 0 }

  function handleMouseMove(event: MouseEvent) {
    if (!input || focused) return
    const rect = input.getBoundingClientRect()
    position = { x: event.clientX - rect.left, y: event.clientY - rect.top }
  }

  const handleFocus = () => ([focused, opacity] = [true, 1])
  const handleBlur = () => ([focused, opacity] = [false, 0])
  const handleMouseEnter = () => (opacity = 1)
  const handleMouseLeave = () => (opacity = 0)

  $: unionTransfers = unionTransfersQuery({
    address: address,
    include: ["RECEIVED"],
    refetchInterval: 5_000,
    enabled: !!address && isValidCosmosAddress(address)
  })

  $: newTransfers = $unionTransfers?.data.filter(transfer => Date.parse(transfer.timestamp) > Date.now() - 60_000) ?? []

  const mutation = createMutation({
    mutationKey: ['faucetRequest'],
    mutationFn: async () => getUnoFromFaucet(address),
    onError: (error) => {
      console.error("Error during the faucet request:", error);
    },
    onSuccess: (data) => {
      console.log("Faucet request successful:", data);
    },

  })

  const debouncedSubmit = debounce(() => {
    if (!isValidCosmosAddress(address)) {
      toast.error('Invalid address');
      return;
    }
    submissionStatus = 'submitting'
    $mutation.mutate();
    console.log('here')
    submissionStatus = 'submitted'
  }, debounceDelay);

  const handleSubmit = async () => {
    debouncedSubmit();
  };

  $: console.log($mutation)
  $: if ($mutation.status === 'success') toast.success('Success!')

  $: unionBalancesQuery = createQuery<Balance>({
    queryKey: [$cosmosStore.address, "balance", "union-testnet-8"],
    refetchInterval: 5000,
    queryFn: async () => {
      const response = await fetch(
        `https://union-testnet-api.polkachu.com/cosmos/bank/v1beta1/balances/${$cosmosStore.address}`
      );

      if (!response.ok) {
        return { amount: "0.00", denom: "muno" }
      }

      const data = await response.json() as {
        balances: Array<{ amount: string; denom: string }>
      };

      const munoBalance = data.balances.find(balance => balance.denom === 'muno');
      return munoBalance || { denom: 'muno', amount: '0.00' };
    },
    enabled: isValidCosmosAddress($cosmosStore.address)
  });

</script>

<svelte:head>
  <title>Union | Faucet</title>
</svelte:head>

<main class="flex justify-center items-start max-h-full px-0 sm:px-3 py-8">
  <Card.Root class="max-w-[475px] w-full">
    <Card.Header>
      <Card.Title class="font-bold text-2xl" tag="h1">Faucet</Card.Title>
    </Card.Header>
    <Card.Content>
      <form class="space-y-8" on:submit|preventDefault={handleSubmit}>
        <div class="relative flex flex-col gap-4">
          <div class="grid w-full items-center gap-2 mb-4">
            <Label for="address">Address</Label>
            <div class="flex items-start gap-2 ">
              <div class="w-full">
                <div class="relative w-full mb-2">
                  <Input
                    autocapitalize="none"
                    autocomplete="off"
                    autocorrect="off"
                    bind:value={address}
                    disabled={inputState === 'locked'}
                    id="address"
                    on:blur={handleInputCommit}
                    on:focus={handleFocus}
                    on:input={handleInput}
                    on:mouseenter={handleMouseEnter}
                    on:mouseleave={handleMouseLeave}
                    on:mousemove={handleMouseMove}
                    pattern={unionAddressRegex.source}
                    placeholder="union14ea6..."
                    required={true}
                    spellcheck="false"
                    type="text"
                  />
                  <input
                    aria-hidden="true"
                    bind:this={input}
                    class="pointer-events-none absolute left-0 top-0 z-10 h-10 sm:h-11 w-full cursor-default rounded-md border border-[#8678F9] bg-[transparent] p-3.5 opacity-0 transition-opacity duration-500 placeholder:select-none"
                    disabled
                    name="style-input-mask"
                    style={`opacity: ${opacity};mask-image: radial-gradient(30% 30px at ${position.x}px ${position.y}px, black 80%, transparent);`}
                  />
                </div>
                <div class="flex justify-between px-1">
                  {#if $unionBalancesQuery.data}
                    <p class="text-xs text-muted-foreground">
                      Balance: {parseInt($unionBalancesQuery.data.amount) / 1000000}</p>
                  {:else}
                    <p class="text-xs text-muted-foreground">Balance: <span class="font-bold">0</span></p>
                  {/if}
                  {#if userInput}
                    <button type="button" on:click={resetInput}
                            class="text-xs text-muted-foreground hover:text-primary transition">Reset
                    </button>
                  {/if}
                </div>
              </div>
              <Button aria-label="Toggle address lock" class="px-3" on:click={onLockClick}
                      variant="ghost">
                {#if inputState === 'locked'}
                  <LockLockedIcon size="medium"/>
                {:else}
                  <LockOpenIcon size="medium"/>
                {/if}
              </Button>
            </div>
          </div>
          <div class="flex flex-col gap-4 sm:flex-row">
            <Button class="w-full sm:w-fit" type="submit">
              Submit
              {#if submissionStatus === 'submitting'}
                <LoadingIcon/>
              {/if}
            </Button>
            <a class="flex items-center gap-x-2 font-bold text-xs" href="https://git-faucets.web.val.run"
               rel="noopener noreferrer"
               target="_blank">
              Faucets for other assets & chains
              <ExternalLinkIcon size="small"/>
            </a>
          </div>
        </div>
      </form>
    </Card.Content>
  </Card.Root>
</main>

<style lang="postcss">
</style>

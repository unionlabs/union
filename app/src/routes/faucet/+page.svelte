<script lang="ts">
import toast from "svelte-french-toast"
import { cn } from "$lib/utilities/shadcn.ts"
import { debounce } from "$lib/utilities/index.ts"
import LockLockedIcon from "virtual:icons/lucide/lock"
import { valibot } from "sveltekit-superforms/adapters"
import * as Form from "$lib/components/ui/form/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import LockOpenIcon from "virtual:icons/lucide/lock-open"
import { superForm, defaults } from "sveltekit-superforms"
import { Button } from "$lib/components/ui/button/index.ts"
import { getUnoFromFaucet } from "$lib/mutations/faucet.ts"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import LoadingIcon from "virtual:icons/lucide/loader-circle"
import * as flashModule from "sveltekit-flash-message/client"
import { unionTransfersQuery } from "$lib/queries/transfers.ts"
import ExternalLinkIcon from "virtual:icons/lucide/external-link"
import { faucetFormSchema, unionAddressRegex } from "./schema.ts"
import { Separator } from "$lib/components/ui/separator/index.ts"
import { isValidCosmosAddress } from "$/lib/wallet/utilities/validate.ts"
import * as Card from "$lib/components/ui/card/index.ts"

const debounceDelay = 3_500
let submissionStatus: "idle" | "submitting" | "submitted" | "error" = "idle"

$: {
  if (submissionStatus === "submitting") {
    toast.loading("Submitting faucet request 🚰", {
      duration: debounceDelay - 300,
      className: "text-sm p-2.5"
    })
  }
}

let inputState: "locked" | "unlocked" = $cosmosStore.address ? "locked" : "unlocked"
const onLockClick = () => (inputState = inputState === "locked" ? "unlocked" : "locked")

const superFormResults = superForm(
  defaults({ address: $cosmosStore.address ?? "" }, valibot(faucetFormSchema)),
  {
    SPA: true,
    validators: valibot(faucetFormSchema),
    flashMessage: {
      module: flashModule,
      onError: event => event.flashMessage.set(event.result.error.message)
    },
    onSubmit: input => {
      submissionStatus = "submitting"
      debounce(async () => {
        if (!$form.address) input.cancel()
        try {
          console.log("Submitting faucet request")
          const result = await getUnoFromFaucet($form.address)
          console.log("Faucet request submitted", result)
          submissionStatus = "submitted"
        } catch (error) {
          submissionStatus = "error"
        }
      }, debounceDelay)()
    },
    onUpdate: event =>
      debounce(() => {
        if (!event.form.valid) {
          $errors.address = event.form.errors.address
          return toast.error("No good", { className: "font-mono text-lg" })
        }

        toast.success("Faucet request submitted 🤌 Check wallet for $U in a few moments", {
          duration: 5_000,
          className: "text-sm p-2.5"
        })
      }, debounceDelay)(),
    delayMs: 7_500,
    timeoutMs: 10_000,
    resetForm: false,
    multipleSubmits: "prevent",
    autoFocusOnError: "detect",
    clearOnSubmit: "errors-and-message"
  }
)

const { enhance, message, delayed, errors, submitting, form } = superFormResults

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
  address: $form.address,
  include: ["RECEIVED"],
  refetchInterval: 5_000,
  enabled: !!$form.address && isValidCosmosAddress($form.address)
})

$: newTransfers =
  $unionTransfers?.data.filter(transfer => Date.parse(transfer.timestamp) > Date.now() - 60_000) ??
  []
</script>

<svelte:head>
  <title>Union | Faucet</title>
</svelte:head>

<main
  class="overflow-scroll flex justify-center size-full items-start px-0 sm:px-3 max-h-full sm:py-8"
>
  <Card.Root class={cn("max-w-[475px] w-full")}>
    <Card.Header>
      <Card.Title tag="h1" class="flex-1 font-bold text-2xl">Faucet</Card.Title>
    </Card.Header>
    <Card.Content>
  <form
    use:enhance
    method="POST"
    class={cn(
      'space-y-8 max-w-[580px]',
      $cosmosStore.address ? 'sm:w-[485px] w-[380px]' : 'sm:w-[460px] w-[400px]',
      'px-1',
      ($delayed || $submitting || $message?.status === 'success') && 'invisible',
    )}
  >
    <Form.Field form={superFormResults} name="address">
      <Form.Control let:attrs>
        <Form.Label class="sm:text-lg text-md">Address</Form.Label>
        <div class="relative">
          <Input
            {...attrs}
            type="text"
            name="address"
            required={true}
            autocorrect="off"
            autocomplete="off"
            spellcheck="false"
            autocapitalize="none"
            on:blur={handleBlur}
            on:focus={handleFocus}
            on:input={event => {
              console.log('Input event', event)
            }}
            bind:value={$form.address}
            on:mousemove={handleMouseMove}
            on:mouseleave={handleMouseLeave}
            on:mouseenter={handleMouseEnter}
            pattern={unionAddressRegex.source}
            aria-invalid={$errors.address ? 'true' : 'false'}
            disabled={$submitting || inputState === 'locked'}
            placeholder="union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
          />
            <!-- Commented out for now because broken on light mode !-->
            <!-- class={cn(
              'peer',
              submissionStatus === 'submitting' && 'animate-pulse',
              'font-mono sm:text-md text-xs w-full h-10 sm:h-11 disabled:opacity-90 disabled:bg-stone-950',
              'rounded-md border border-slate-800 bg-neutral-950 p-3.5 text-slate-100 transition-colors placeholder:select-none placeholder:text-neutral-600 focus:border-[#8678F9]',
              'focus:outline-none outline-transparent focus-visible:outline-none ring-0 focus:ring-0 focus-visible:ring-0',
            )} !-->
          <input
            disabled
            bind:this={input}
            aria-hidden="true"
            name="style-input-mask"
            style={`
              opacity: ${opacity};
              mask-image: radial-gradient(30% 30px at ${position.x}px ${position.y}px, black 80%, transparent);
            `}
            class="pointer-events-none absolute left-0 top-0 z-10 h-10 sm:h-11 w-full cursor-default rounded-md border border-[#8678F9] bg-[transparent] p-3.5 opacity-0 transition-opacity duration-500 placeholder:select-none"
          />
          {#if $cosmosStore.address}
            <Button
              size="icon"
              type="button"
              variant="ghost"
              name="recipient-lock"
              on:click={onLockClick}
              class="absolute bottom-0 sm:bottom-[2px] right-0 rounded-l-none"
            >
              <LockLockedIcon
                class={cn(
                  inputState === 'locked' && $form.address.length > 0 ? 'size-5' : 'hidden',
                )}
              />
              <LockOpenIcon
                class={cn(inputState === 'unlocked' || !$form.address.length ? 'size-5' : 'hidden')}
              />
            </Button>
          {/if}
        </div>
      </Form.Control>
      <Form.FieldErrors class="field-errors peer" />
      <Form.Description class="block peer-[&:not(:empty)]:hidden sm:text-sm text-xs ml-2">
        A valid Union wallet address
      </Form.Description>
    </Form.Field>
    <div class="flex gap-x-2 w-full">
      <!-- Commented out for now because broken on light mode !-->
      <!-- class={cn(
        submissionStatus === 'submitting' && 'animate-pulse',
        'sm:text-md text-sm font-bold w-full sm:max-w-32 max-w-20 tracking-wider ml-1',
      )} !-->
      <Form.Button
        disabled={$submitting || $form.address.length === 0}
      >
        Submit
        <LoadingIcon
          class={cn(submissionStatus === 'submitting' ? 'animate-spin ml-2 size-5' : 'hidden')}
        />
      </Form.Button>
      <Separator orientation="vertical" class="bg-stone-600 h-7 my-auto" />
      <Button
        variant="link"
        target="_blank"
        rel="noopener noreferrer"
        href="https://git-faucets.web.val.run"
        class="sm:text-sm text-xs font-bold px-0 my-auto !text-neutral-300"
      >
        Faucets for other assets & chains
        <ExternalLinkIcon class="size-4 ml-2" />
      </Button>
    </div>
  </form>
    </Card.Content>
  </Card.Root>
</main>

<style lang="postcss">
</style>

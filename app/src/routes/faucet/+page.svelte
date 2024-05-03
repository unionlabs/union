<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

import toast from "svelte-french-toast"
import { browser } from "$app/environment"
import { valibot } from "sveltekit-superforms/adapters"
import * as Form from "$lib/components/ui/form/index.js"
import { Input } from "$lib/components/ui/input/index.js"
import { getUnoFromFaucet } from "$lib/mutations/faucet.ts"
import { unionTransfersQuery } from "$lib/queries/transfers.ts"
import { faucetFormSchema, unionAddressRegex } from "./schema.ts"
import DraftPageNotice from "$lib/components/draft-page-notice.svelte"
import { superForm, setError, setMessage, defaults } from "sveltekit-superforms"

/**
 * TODO:
 * [ ] - Display user received transactions & show spinner while loading
 */

// export let data: PageData

const form = superForm(defaults(valibot(faucetFormSchema)), {
  SPA: true,
  validators: valibot(faucetFormSchema),
  onUpdate: event => {
    if (!event.form.valid) return toast.error("No good", { className: "font-mono text-lg" })

    toast.success("Faucet request submitted 🤌 Check wallet for $UNO in a few moments", {
      duration: 5_000,
      className: "text-sm p-2.5"
    })
  },
  multipleSubmits: "prevent"
})

const { enhance, message, delayed, errors, submitting, form: formData } = form

$: unionTransfers = unionTransfersQuery({
  address: $formData.address,
  include: ["RECEIVED"],
  refetchInterval: 5_000
})

$: newTransfers =
  $unionTransfers?.data.filter(transfer => Date.parse(transfer.timestamp) > Date.now() - 60_000) ??
  []

let input: HTMLInputElement
let focused = false
let position = { x: 0, y: 0 }
let opacity = 0

function handleMouseMove(event: MouseEvent) {
  if (!input || focused) return
  const rect = input.getBoundingClientRect()
  position = { x: event.clientX - rect.left, y: event.clientY - rect.top }
}

const handleFocus = () => {
  ;[focused, opacity] = [true, 1]
}
const handleBlur = () => {
  ;[focused, opacity] = [false, 0]
}
const handleMouseEnter = () => {
  opacity = 1
}
const handleMouseLeave = () => {
  opacity = 0
}
</script>

<svelte:head>
  <title>Union - Faucet</title>
</svelte:head>

<main class="mx-auto w-full flex flex-col items-center px-4 mt-16">
  <DraftPageNotice />

  <h1 class="text-3xl font-extrabold my-6">Faucet</h1>
  {#if $delayed || $submitting}
    LOADING
  {/if}
  {#if $message?.status === 'success'}
    <div>{$message.text}</div>
  {/if}
  {#if $errors._errors?.length}
    <div>{JSON.stringify($errors, undefined, 2)}</div>
  {/if}
  <form
    use:enhance
    method="POST"
    class={cn([
      'sm:w-[400px] w-[350px] max-w-[500px] space-y-6',
      ($delayed || $submitting || $message?.status === 'success') && 'invisible',
    ])}
  >
    <Form.Field {form} name="address">
      <Form.Control let:attrs>
        <Form.Label>Address</Form.Label>
        <div class="relative">
          <Input
            {...attrs}
            spellcheck={false}
            bind:value={$formData.address}
            pattern={unionAddressRegex.source}
            on:focus={handleFocus}
            on:blur={handleBlur}
            on:mouseenter={handleMouseEnter}
            on:mouseleave={handleMouseMove}
            placeholder="union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
            class="h-12 w-full cursor-default rounded-md border border-slate-800 bg-neutral-950 p-3.5 text-slate-100 transition-colors duration-500 placeholder:select-none placeholder:text-neutral-600 focus:border-[#8678F9] focus:outline-none"
            title="Must be a valid Union address (bech32, starts with `union`)"
          />
          <input
            bind:this={input}
            disabled
            aria-hidden="true"
            style={`
              opacity: ${opacity};
              -webkit-mask-image: radial-gradient(30% 30px at ${position.x}px ${position.y}px, black 80%, transparent);
            `}
            class="pointer-events-none absolute left-0 top-0 z-10 h-12 w-full cursor-default rounded-md border border-[#8678F9] bg-[transparent] p-3.5 opacity-0 transition-opacity duration-500 placeholder:select-none"
          />
        </div>
      </Form.Control>
      <Form.Description>A valid Union wallet address.</Form.Description>
      <Form.FieldErrors />
    </Form.Field>
    <Form.Button>Submit</Form.Button>
  </form>

  {#if $unionTransfers?.status === 'success'}
    {#each newTransfers as transfer}
      <pre>{transfer?.timestamp}</pre>
    {/each}
  {/if}

  <section class="mt-6 hidden sm:block w-full max-w-[520px] text-sm">
  </section>
</main>

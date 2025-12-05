<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { Ucs05 } from "@unionlabs/sdk"
import { Bech32FromAddressCanonicalBytesWithPrefix, Chain } from "@unionlabs/sdk/schema"
import * as Address from "@unionlabs/sdk/schema/address"
import {
  Array as A,
  Either as E,
  Match,
  Option as O,
  ParseResult,
  pipe,
  Schema as S,
  Struct,
} from "effect"
import { constFalse, constVoid, flow } from "effect/Function"
import { onMount } from "svelte"
import type { FormEventHandler } from "svelte/elements"

type Props = {
  label?: string | undefined
  chain: O.Option<Chain>
  type: "sender" | "receiver"
  address: string | undefined
  onValid: (address: string, encoded: string) => void
  onError: (error: string[]) => void
}

const _props: Props = $props()

const props = $derived(
  Struct.evolve(_props, {
    address: (x) => O.fromNullable(x),
  }),
)

let inputValue = $state<string>("")

$effect(() => {
  const next = O.getOrElse(props.address, () => "" as const)
  if (next !== "" && next !== inputValue) {
    inputValue = next
  }
})

const placeholder = $derived(
  pipe(
    props.chain,
    O.map((x) => `${x.addr_prefix}...`),
    O.getOrElse(() => "..."),
  ),
)

const transform = $derived(
  Match.value({ chain: props.chain, type: props.type }).pipe(
    Match.when(
      { chain: O.isSome, type: "sender" },
      ({ chain }) =>
        S.encodeUnknownEither(
          Bech32FromAddressCanonicalBytesWithPrefix(chain.value.addr_prefix),
        ),
    ),
    Match.when(
      { chain: O.isSome, type: "receiver" },
      ({ chain }) =>
        pipe(
          Match.value(chain.value.rpc_type),
          Match.when("evm", () => S.validateEither(Address.ERC55)),
          Match.when(
            "cosmos",
            () =>
              S.encodeUnknownEither(
                Bech32FromAddressCanonicalBytesWithPrefix(chain.value.addr_prefix),
              ),
          ),
          Match.when("sui", () => S.validateEither(Ucs05.SuiAddress)),
          Match.when("aptos", () => S.validateEither(Address.AddressAptosDisplay)),
          Match.orElseAbsurd,
        ),
    ),
    Match.option,
  ),
)

const disabled = $derived(
  pipe(
    Match.value(props.type),
    Match.when("sender", constFalse),
    Match.when("receiver", () => O.isNone(transform)),
    Match.exhaustive,
  ),
)

const validateAddress = (address: string) => {
  if (address === "") {
    return
  }

  return pipe(
    transform,
    O.match({
      onNone: constVoid,
      onSome: (fn) =>
        pipe(
          fn(address),
          E.match({
            onLeft: flow(
              ParseResult.ArrayFormatter.formatErrorSync,
              A.map((x) => x.message),
              props.onError,
            ),
            onRight: (encoded) => props.onValid(address, encoded),
          }),
        ),
    }),
  )
}

$effect(() => {
  void transform
  if (inputValue !== "") {
    validateAddress(inputValue)
  }
})

onMount(() => {
  const atStart = O.getOrElse(props.address, () => "" as const)
  if (atStart !== "") {
    inputValue = atStart
    validateAddress(inputValue)
  }
})

const onInput: FormEventHandler<HTMLInputElement> = (event) => {
  inputValue = event.currentTarget.value
  validateAddress(inputValue)
}
</script>

<Input
  label={props.label ?? ""}
  id={props.type}
  type="text"
  disabled={disabled}
  required
  autocorrect="off"
  placeholder={placeholder}
  spellcheck="false"
  autocomplete="off"
  inputmode="text"
  autocapitalize="none"
  value={inputValue}
  class="h-14 text-center text-lg focus:text-white"
  oninput={onInput}
/>

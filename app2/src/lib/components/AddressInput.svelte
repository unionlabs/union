<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { runSync } from "$lib/runtime"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { Bech32FromAddressCanonicalBytesWithPrefix, Chain } from "@unionlabs/sdk/schema"
import * as Address from "@unionlabs/sdk/schema/address"
import {
  Array as A,
  Effect,
  Either as E,
  Match,
  Option as O,
  ParseResult,
  pipe,
  Schema as S,
  Struct,
} from "effect"
import { apply, constFalse, constVoid, flow } from "effect/Function"
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

const props = $derived(Struct.evolve(_props, {
  address: (x) => O.fromNullable(x),
}))

const placeholder = $derived(
  Match.value(props.type).pipe(
    Match.when("sender", () => "bbn..."),
    Match.when("receiver", () =>
      pipe(
        props.chain,
        O.map(x => `${x.addr_prefix}...`),
        O.getOrElse(() => "..."),
      )),
    Match.exhaustive,
  ),
)

const transform = $derived(
  Match.value(props.type).pipe(
    Match.when(
      "sender",
      () =>
        O.some(
          S.encodeUnknownEither(
            Bech32FromAddressCanonicalBytesWithPrefix("bbn"),
          ),
        ),
    ),
    Match.when("receiver", () =>
      O.map(
        props.chain,
        (chain) =>
          pipe(
            Match.value(chain.rpc_type),
            Match.when("evm", () => S.validateEither(Address.ERC55)),
            Match.when(
              "cosmos",
              () =>
                S.encodeUnknownEither(
                  Bech32FromAddressCanonicalBytesWithPrefix(chain.addr_prefix),
                ),
            ),
            Match.orElseAbsurd,
          ),
      )),
    Match.exhaustive,
  ),
)

const validateAddress = (address: string) =>
  pipe(
    transform,
    O.map(
      flow(
        apply(address),
        E.match({
          onLeft: flow(
            ParseResult.ArrayFormatter.formatErrorSync,
            A.map(x => x.message),
            props.onError,
          ),
          onRight: (encoded) => props.onValid(address, encoded),
        }),
      ),
    ),
    O.getOrElse(constVoid),
  )

const disabled = $derived(pipe(
  Match.value(props.type),
  Match.when("sender", constFalse),
  Match.when("receiver", () => O.isNone(transform)),
  Match.exhaustive,
))

const display = $derived(O.getOrElse(props.address, () => "" as const))

const onInput: FormEventHandler<HTMLInputElement> = event =>
  validateAddress(event.currentTarget.value)

onMount(() => {
  if (O.isSome(props.address)) {
    validateAddress(props.address.value)
  }
})
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
  value={display}
  class="h-14 text-center text-lg focus:text-white"
  oninput={onInput}
/>

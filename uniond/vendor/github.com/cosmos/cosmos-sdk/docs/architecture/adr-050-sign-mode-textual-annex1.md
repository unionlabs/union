# ADR 050: SIGN_MODE_TEXTUAL: Annex 1 Value Renderers

## Changelog

- Dec 06, 2021: Initial Draft
- Feb 07, 2022: Draft read and concept-ACKed by the Ledger team.

## Status

Accepted. Implementation started. Small value renderers details still need to be polished.

## Abstract

This Annex describes value renderers, which are used for displaying Protobuf values in a human-friendly way using a string array.

## Value Renderers

Value Renderers describe how values of different Protobuf types should be encoded as a string array. Value renderers can be formalized as a set of bijective functions `func renderT(value T) []string`, where `T` is one of the below Protobuf types for which this spec is defined.

### Protobuf `number`

- Applies to:
  - protobuf numeric integer types (`int{32,64}`, `uint{32,64}`, `sint{32,64}`, `fixed{32,64}`, `sfixed{32,64}`)
  - strings whose `customtype` is `github.com/cosmos/cosmos-sdk/types.Int` or `github.com/cosmos/cosmos-sdk/types.Dec`
  - bytes whose `customtype` is `github.com/cosmos/cosmos-sdk/types.Int` or `github.com/cosmos/cosmos-sdk/types.Dec`
- Trailing decimal zeroes are always removed
- Formatting with `'`s for every three integral digits.
- Usage of `.` to denote the decimal delimiter.

#### Examples

- `1000` (uint64) -> `1'000`
- `"1000000.00"` (string representing a Dec) -> `1'000'000`
- `"1000000.10"` (string representing a Dec) -> `1'000'000.1`

### `coin`

- Applies to `cosmos.base.v1beta1.Coin`.
- Denoms are converted to `display` denoms using `Metadata` (if available). **This requires a state query**. The definition of `Metadata` can be found in the [bank Protobuf definition](https://github.com/cosmos/cosmos-sdk/blob/v0.46.0/proto/cosmos/bank/v1beta1/bank.proto#L79-L108). If the `display` field is empty or nil, then we do not perform any denom conversion.
- Amounts are converted to `display` denom amounts and rendered as `number`s above
  - We do not change the capitalization of the denom. In practice, `display` denoms are stored in lowercase in state (e.g. `10 atom`), however they are often showed in UPPERCASE in everyday life (e.g. `10 ATOM`). Value renderers keep the case used in state, but we may recommend chains changing the denom metadata to be uppercase for better user display.
- One space between the denom and amount (e.g. `10 atom`).
- In the future, IBC denoms could maybe be converted to DID/IIDs, if we can find a robust way for doing this (ex. `cosmos:cosmos:hub:bank:denom:atom`)

#### Examples

- `1000000000uatom` -> `["1'000 atom"]`, because atom is the metadata's display denom.

### `coins`

- an array of `coin` is display as the concatenation of each `coin` encoded as the specification above, the joined together with the delimiter `", "` (a comma and a space, no quotes around).
- the list of coins is ordered by unicode code point of the display denom: `A-Z` < `a-z`. For example, the string `aAbBcC` would be sorted `ABCabc`.

### Example

- `["3cosm", "2000000uatom"]` -> `2 atom, 3 COSM` (assuming the display denoms are `atom` and `COSM`)
- `["10atom", "20Acoin"]` -> `20 Acoin, 10 atom` (assuming the display denoms are `atom` and `Acoin`)

### `repeated`

- Applies to all `repeated` fields, except `cosmos.tx.v1beta1.TxBody#Messages`, which has a particular encoding (see [ADR-050](./adr-050-sign-mode-textual.md)).
- A repeated type has the following template:

```
<message_name> has <int> <field_name>
<field_name> (<int>/<int>): <value rendered 1st line>
<optional value rendered in the next lines>
<field_name> (<int>/<int>): <value rendered 1st line>
<optional value rendered in the next lines>
End of <field_name>.
```

where:

- `message_name` is the name of the Protobuf message which holds the `repeated` field,
- `int` is the length of the array,
- `field_name` is the Protobuf field name of the repeated field,
  - add an optional `s` at the end if `<int> > 1` and the `field_name` doesn't already end with `s`.

#### Examples

Given the proto definition:

```protobuf
message AllowedMsgAllowance {
  repeated string allowed_messages = 1;
}
```

and initializing with:

```go
x := []AllowedMsgAllowance{"cosmos.bank.v1beta1.MsgSend", "cosmos.gov.v1.MsgVote"}
```

we have the following value-rendered encoding:

```
Allowed messages: 2 strings
Allowed messages (1/2): cosmos.bank.v1beta1.MsgSend
Allowed messages (2/2): cosmos.gov.v1.MsgVote
End of Allowed messages
```

### `message`

- Applies to Protobuf messages whose name does not start with `Msg`
  - For `sdk.Msg`s, please see [ADR-050](./adr-050-sign-mode-textual.md)
  - alternatively, we can decide to add a protobuf option to denote messages that are `sdk.Msg`s.
- Field names follow [sentence case](https://en.wiktionary.org/wiki/sentence_case)
  - replace `_` with a spaces
  - capitalize first letter of the setence
- Field names are ordered by their Protobuf field number
- Nesting:
  - if a field contains a nested message, we value-render the underlying message using the template:
  ```
  <field_name>: <1st line of value-rendered message>
  > <lines 2-n of value-rendered message>             // Notice the `>` prefix.
  ```
  - `>` character is used to denote nesting. For each additional level of nesting, add `>`.

#### Examples

Given the following Protobuf messages:

```protobuf
enum VoteOption {
  VOTE_OPTION_UNSPECIFIED = 0;
  VOTE_OPTION_YES = 1;
  VOTE_OPTION_ABSTAIN = 2;
  VOTE_OPTION_NO = 3;
  VOTE_OPTION_NO_WITH_VETO = 4;
}

message WeightedVoteOption {
  VoteOption option = 1;
  string     weight = 2 [(cosmos_proto.scalar) = "cosmos.Dec"];
}

message Vote {
  uint64 proposal_id = 1;
  string voter       = 2 [(cosmos_proto.scalar) = "cosmos.AddressString"];
  reserved 3;
  repeated WeightedVoteOption options = 4;
}
```

we get the following encoding for the `Vote` message:

```
Vote object
> Proposal id: 4
> Vote: cosmos1abc...def
> Options: 2 WeightedVoteOptions
> Options (1/2): WeightedVoteOption object
>> Option: Yes
>> Weight: 0.7
> Options (2/2): WeightedVoteOption object
>> Option: No
>> Weight: 0.3
> End of Options
```

### Enums

- String case convention: snake case to sentence case
- Allow optional annotation for textual name (TBD)
- Algorithm:
  - convert enum name (`VoteOption`) to snake_case (`VOTE_OPTION`)
  - truncate that prefix + `_` from the enum name if it exists (`VOTE_OPTION_` gets stripped from `VOTE_OPTION_YES` -> `YES`)
  - convert rest to sentence case: `YES` -> `Yes`
  - in summary: `VOTE_OPTION_YES` -> `Yes`

#### Examples

See example above with `message Vote{}`.

### `google.protobuf.Any`

- Applies to `google.protobuf.Any`
- Rendered as:

```
Object: <type_url>
> <value rendered underlying message>
```

#### Examples

```
Object: /cosmos.gov.v1.Vote
> Proposal id: 4
> Vote: cosmos1abc...def
> Options: 2 WeightedVoteOptions
> Options (1/2): WeightedVoteOption object
>> Option: Yes
>> Weight: 0.7
> Options (2/2): WeightedVoteOption object
>> Option: No
>> Weight: 0.3
> End of Options
```

### `google.protobuf.Timestamp`

Rendered using [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) (a
simplification of ISO 8601), which is the current recommendation for portable
time values. The rendering always uses "Z" (UTC) as the timezone. It uses only
the necessary fractional digits of a second, omitting the fractional part
entirely if the timestamp has no fractional seconds. (The resulting timestamps
are not automatically sortable by standard lexicographic order, but we favor
the legibility of the shorter string.)

#### Examples

The timestamp with 1136214245 seconds and 700000000 nanoseconds is rendered
as `2006-01-02T15:04:05.7Z`.
The timestamp with 1136214245 seconds and zero nanoseconds is rendered
as `2006-01-02T15:04:05Z`.

### `google.protobuf.Duration`

The duration proto expresses a raw number of seconds and nanoseconds.
This will be rendered as longer time units of days, hours, and minutes,
plus any remaining seconds, in that order.
Leading and trailing zero-quantity units will be omitted, but all
units in between nonzero units will be shown, e.g. ` 3 days, 0 hours, 0 minutes, 5 seconds`.

Even longer time units such as months or years are imprecise.
Weeks are precise, but not commonly used - `91 days` is more immediately
legible than `13 weeks`.  Although `days` can be problematic,
e.g. noon to noon on subsequent days can be 23 or 25 hours depending on
daylight savings transitions, there is significant advantage in using
strict 24-hour days over using only hours (e.g. `91 days` vs `2184 hours`).

When nanoseconds are nonzero, they will be shown as fractional seconds,
with only the minimum number of digits, e.g `0.5 seconds`.

A duration of exactly zero is shown as `0 seconds`.

Units will be given as singular (no trailing `s`) when the quantity is exactly one,
and will be shown in plural otherwise.

Negative durations will be indicated with a leading minus sign (`-`).

Examples:

- `1 day`
- `30 days`
- `-1 day, 12 hours`
- `3 hours, 0 minutes, 53.025 seconds`

### bytes

- Bytes are rendered in hexadecimal, all capital letters, without the `0x` prefix.

### address bytes

We currently use `string` types in protobuf for addresses so this may not be needed, but if any address bytes are used in sign mode textual they should be rendered with bech32 formatting

### strings

Strings are rendered as-is.

### Default Values

- Default Protobuf values for each field are skipped.

#### Example

```protobuf
message TestData {
  string signer = 1;
  string metadata = 2;
}
```

```go
myTestData := TestData{
  Signer: "cosmos1abc"
}
```

We get the following encoding for the `TestData` message:

```
TestData object
> Signer: cosmos1abc
```

### [ABANDONED] Custom `msg_title` instead of Msg `type_url`

_This paragraph is in the Annex for informational purposes only, and will be removed in a next update of the ADR._

<details>
  <summary>Click to see abandoned idea.</summary>

- all protobuf messages to be used with `SIGN_MODE_TEXTUAL` CAN have a short title associated with them that can be used in format strings whenever the type URL is explicitly referenced via the `cosmos.msg.v1.textual.msg_title` Protobuf message option.
- if this option is not specified for a Msg, then the Protobuf fully qualified name will be used.

```protobuf
message MsgSend {
  option (cosmos.msg.v1.textual.msg_title) = "bank send coins";
}
```

- they MUST be unique per message, per chain

#### Examples

- `cosmos.gov.v1.MsgVote` -> `governance v1 vote`

#### Best Pratices

We recommend to use this option only for `Msg`s whose Protobuf fully qualified name can be hard to understand. As such, the two examples above (`MsgSend` and `MsgVote`) are not good examples to be used with `msg_title`. We still allow `msg_title` for chains who might have `Msg`s with complex or non-obvious names.

In those cases, we recommend to drop the version (e.g. `v1`) in the string if there's only one version of the module on chain. This way, the bijective mapping can figure out which message each string corresponds to. If multiple Protobuf versions of the same module exist on the same chain, we recommend keeping the first `msg_title` with version, and the second `msg_title` with version (e.g. `v2`):

- `mychain.mymodule.v1.MsgDo` -> `mymodule do something`
- `mychain.mymodule.v2.MsgDo` -> `mymodule v2 do something`

</details>

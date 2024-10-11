---
order: 1
parent:
  title: priv_validator_state.json
  description: Details of last signed block
  order: 5
---
# priv_validator_state.json
When CometBFT is run as a validator and a local private validator (`PrivVal`) is adopted, it uses this file to keep data about the last signed consensus messages.

This file is only updated if a local private validator is adopted.
(When [priv_validator_laddr](config.toml.md#priv_validator_laddr) is not set.)

### Examples
```json
{
  "height": "0",
  "round": 0,
  "step": 0
}
```

```json
{
  "height": "36",
  "round": 0,
  "step": 3,
  "signature": "N813twXq5yC84wKGrD85X79iXPwtVytGdD3j8btwZ5ZyAAHSkNt6NBWvrTJUcMLqefPfG3SBdPHdfOedieeYCg==",
  "signbytes": "76080211240000000000000022480A20D1823B950D1A0FD7335B4E63D2B65CF9D0CEAC13DF4E9E2DFB4765D2C69C74D0122408011220DB69B3B750BBCEAB4BC86BB1847D3E0DDB342EFAFE5731605C61A828265E09802A0C08CDF288AF0610A88CA8FE023211746573742D636861696E2D4866644B6E44"
}
```
## height
Set to the last height that was signed.

| Value type          | string      |
|:--------------------|:------------|
| **Possible values** | &gt;= `"0"` |

Height, a number, is presented as a string so arbitrary high numbers can be used without the limitation of the integer
maximum.

## round
Set to the last round that was signed.

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

## step
Set to the last round step that was signed.

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

## signature
The last signature produced. This was provided at the above [height/round/step](#height).

| Value type          | string               |
|:--------------------|:---------------------|
| **Possible values** | base64-encoded bytes |
|                     | `""`                 |

## signbytes
Proto-encoding of the latest consensus message signed. Used to compare incoming requests and if possible reuse the
previous signature provided in [signature](#signature).

| Value type          | string            |
|:--------------------|:------------------|
| **Possible values** | hex-encoded bytes |
|                     | `""`              |


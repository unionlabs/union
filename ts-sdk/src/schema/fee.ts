import { Array as A, BigDecimal, Equal, Option as O, pipe, Schema as S } from "effect"

export const GasFee = S.BigDecimalFromSelf.pipe(
  S.greaterThanBigDecimal(BigDecimal.make(0n, 0)),
  S.brand("GasFee"),
)
export type GasFee = typeof GasFee.Type

const Action = S.Union(
  S.Literal("PACKET_RECV"),
  S.Literal("PACKET_SEND_LC_UPDATE_L0"),
  S.Literal("PACKET_SEND_LC_UPDATE_L1"),
  S.Literal("PACKET_SEND_LC_UPDATE_L2"),
)
type Action = typeof Action.Type

export const Fees = S.transform(
  pipe(
    S.Struct({
      action: Action,
      fee: S.BigDecimal.pipe(
        S.greaterThanBigDecimal(BigDecimal.make(0n, 0)),
      ),
    }),
    S.Array,
    S.filter(
      (xs) =>
        A.isNonEmptyReadonlyArray(xs)
          ? pipe(
            xs.length,
            Equal.equals(A.dedupeWith(xs, (a, b) => a.action === b.action).length),
          )
          : true,
      {
        description: "Fees must be unique by action if non-empty.",
      },
    ),
  ),
  S.Struct({
    PACKET_RECV: S.OptionFromSelf(GasFee),
    PACKET_SEND_LC_UPDATE_L0: S.OptionFromSelf(GasFee),
    PACKET_SEND_LC_UPDATE_L1: S.OptionFromSelf(GasFee),
    PACKET_SEND_LC_UPDATE_L2: S.OptionFromSelf(GasFee),
  }),
  {
    decode: (fromA) => {
      const byAction = <T extends Action>(action: T) =>
        pipe(
          A.findFirst(fromA, x => x.action === action),
          O.map(x => x.fee),
        )
      return {
        PACKET_RECV: byAction("PACKET_RECV"),
        PACKET_SEND_LC_UPDATE_L0: byAction("PACKET_SEND_LC_UPDATE_L0"),
        PACKET_SEND_LC_UPDATE_L1: byAction("PACKET_SEND_LC_UPDATE_L1"),
        PACKET_SEND_LC_UPDATE_L2: byAction("PACKET_SEND_LC_UPDATE_L2"),
      }
    },
    encode: (toI) =>
      pipe(
        toI,
        A.fromRecord,
        A.map(x => ({ action: x[0], fee: x[1] })),
        (xs) => xs as unknown as any, // XXX: wrong
      ),
    strict: true,
  },
)
export type Fees = typeof Fees.Type

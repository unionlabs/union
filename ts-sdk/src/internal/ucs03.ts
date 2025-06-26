import * as Ucs03 from "../Ucs03.js"

type EnsureExported = Extract<(typeof Ucs03.Abi)[number], { name: "ensureExported" }>

type StructMap = {
  [
    P in EnsureExported["inputs"][number] as P["internalType"] extends
      `struct ${infer S extends string}` ? S
      : never
  ]: P["components"]
}

/** @internal */
export const fromStruct = <const S extends keyof StructMap>(name: S): StructMap[S] =>
  (Ucs03.Abi as readonly unknown[])
    .find((i): i is EnsureExported =>
      typeof i === "object" && i !== null && (i as any).name === "ensureExported"
    )!
    .inputs
    .find((i): i is Extract<EnsureExported["inputs"][number], { internalType: `struct ${S}` }> =>
      (i as any).internalType === `struct ${name}`
    )!
    .components as StructMap[S]

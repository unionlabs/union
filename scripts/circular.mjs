/* eslint-disable no-undef */
import * as glob from "glob"
import madge from "madge"

madge(
  glob.globSync([
    "ts-sdk/*/src/**/*.ts",
    "ts-sdk-evm/*/src/**/*.ts",
    "ts-sdk-cosmos/*/src/**/*.ts",
  ], {
    ignore: [],
  }),
  {
    detectiveOptions: {
      ts: {
        skipTypeImports: true,
      },
    },
  },
).then((res) => {
  const circular = res.circular()
  if (circular.length) {
    console.error("Circular dependencies found")
    console.error(circular)
    process.exit(1)
  }
})

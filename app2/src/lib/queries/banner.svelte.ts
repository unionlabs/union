import { bannerStore } from "$lib/stores/banner.svelte"
import { createQuery } from "$lib/utils/queries"
import { Schema } from "effect"

const BannerConfig = Schema.Struct({
  enabled: Schema.Boolean,
  type: Schema.Literal("info", "warning", "error"),
  message: Schema.String,
})

const BannerData = Schema.Struct({
  app: Schema.Struct({
    banner: BannerConfig,
  }),
  btc: Schema.Struct({
    banner: BannerConfig,
  }),
})

export const bannerQuery = () =>
  createQuery({
    url: "https://app-banner.unionlabs.workers.dev/",
    schema: BannerData,
    refetchInterval: "1 minutes",
    writeData: data => {
      bannerStore.data = data
    },
    writeError: error => {
      bannerStore.error = error
    },
  })

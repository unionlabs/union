import type { FetchDecodeError } from "$lib/utils/queries"
import { Option } from "effect"

export type BannerType = "info" | "warning" | "error"

export type BannerConfig = {
  enabled: boolean
  type: BannerType
  message: string
}

export type BannerData = {
  app: {
    banner: BannerConfig
  }
  btc: {
    banner: BannerConfig
  }
}

class BannerStore {
  data: Option.Option<BannerData> = $state(Option.none())
  error: Option.Option<FetchDecodeError> = $state(Option.none())

  // Get banner for specific edition
  getBannerForEdition(edition: "app" | "btc"): Option.Option<BannerConfig> {
    return Option.map(this.data, (data) => data[edition].banner)
  }
}

export const bannerStore = new BannerStore()

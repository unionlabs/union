import type { FetchDecodeError } from "$lib/utils/queries"
import { Option } from "effect"

export type BannerType = "info" | "warning" | "error"

export type BannerConfig = {
  readonly enabled: boolean
  readonly type: BannerType
  readonly message: string
}

export type BannerData = {
  readonly app: {
    readonly banners: readonly BannerConfig[]
  }
  readonly btc: {
    readonly banners: readonly BannerConfig[]
  }
}

class BannerStore {
  data: Option.Option<BannerData> = $state(Option.none())
  error: Option.Option<FetchDecodeError> = $state(Option.none())

  // Get banners for specific edition
  getBannersForEdition(edition: "app" | "btc"): BannerConfig[] {
    return Option.match(this.data, {
      onNone: () => [],
      onSome: (data) => data[edition].banners.filter(banner => banner.enabled),
    })
  }
  3
}

export const bannerStore = new BannerStore()

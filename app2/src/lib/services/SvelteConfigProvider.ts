import * as SvelteStaticPublic from "$env/static/public"
import { ConfigProvider, Layer, pipe } from "effect"

export const StaticPublicProvider: ConfigProvider.ConfigProvider = pipe(
  ConfigProvider.fromMap(
    new Map(Object.entries(SvelteStaticPublic)),
    { pathDelim: "_" },
  ),
  ConfigProvider.mapInputPath((s) => `PUBLIC_${s}`),
)

export const StaticPublic = pipe(
  StaticPublicProvider,
  Layer.setConfigProvider,
)

export const SearchParamsProvider: ConfigProvider.ConfigProvider = ConfigProvider.fromMap(
  new Map(new URLSearchParams(window.location.search).entries()),
  { pathDelim: "_" },
)

export const SearchParams: Layer.Layer<never> = pipe(
  SearchParamsProvider,
  Layer.setConfigProvider,
)

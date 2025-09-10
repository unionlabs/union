import { page } from "$app/state"
import base64Icon from "$lib/config/base64.txt?raw"
import { GraphQL } from "$lib/graphql/service"
import * as AppRuntime from "$lib/runtime"
import { themes } from "$lib/themes"
import type { Edition, Theme } from "$lib/themes"
import { Effect, Match, Option, pipe, Record as R, String as Str } from "effect"

const projectIds: Record<Edition, string> = {
  app: "f544d5ee6eb61962408fd456c114e9ed",
  btc: "49fe74ca5ded7142adefc69a7788d14a",
}

const getEditionFromUrl: (_: string) => Edition = Match.type<string>().pipe(
  Match.whenOr(
    Str.startsWith("btc."),
    Str.startsWith("staging.btc."),
    () => "btc" as const,
  ),
  Match.whenOr(
    Str.startsWith("app."),
    Str.startsWith("staging.app."),
    () => "app" as const,
  ),
  Match.orElse(() => "app" as const),
)

type AppInfo = {
  base64Icon: string
  name: string
  baseUrl: string
  docs: string
  iconUrl: string
  projectId: string
}

class UiStore {
  walletModalOpen: boolean = $state(false)
  settingsModalOpen: boolean = $state(false)
  errorsModalOpen: boolean = $state(false)

  showZeroBalances: boolean = $state(false)
  showDeveloperPages: boolean = $state(false)
  filterWhitelist: boolean = $state(true)

  edition: Edition
  overrideEdition: Edition | null = $state(null)

  theme: Theme

  appInfo: AppInfo

  constructor() {
    const overrideEdition = pipe(
      Object.fromEntries(new URLSearchParams(globalThis.location.search)),
      R.get("edition"),
      // TODO: make pretty
      Option.flatMap(Option.liftPredicate(x => ["app", "btc"].includes(x))),
      Option.map(x => x as Edition),
      Option.getOrUndefined,
    )

    this.edition = overrideEdition ?? getEditionFromUrl(globalThis.location.hostname)
    this.theme = themes[this.edition]
    this.appInfo = {
      base64Icon,
      name: "Union",
      baseUrl: `https://${this.edition}.union.build`,
      docs: "https://docs.union.build",
      iconUrl: "https://app.union.build/images/logo.png",
      projectId: projectIds[this.edition],
    }
  }

  private closeAllModals() {
    this.walletModalOpen = false
    this.settingsModalOpen = false
    this.errorsModalOpen = false
  }

  get graphqlEndpoint(): string {
    return AppRuntime.runSync(GraphQL.pipe(
      Effect.andThen((client) => client.getEndpoint),
    ))
  }

  set graphqlEndpoint(s: string) {
    AppRuntime.runPromise(GraphQL.pipe(
      Effect.andThen((client) => client.updateEndpoint(s)),
    ))
  }

  clearGqlCache() {
    AppRuntime.runSync(GraphQL.pipe(
      Effect.andThen((client) => client.resetCache),
    ))
  }

  openWalletModal() {
    this.closeAllModals()
    this.walletModalOpen = true
  }

  closeWalletModal() {
    this.walletModalOpen = false
  }

  openSettingsModal() {
    this.closeAllModals()
    this.settingsModalOpen = true
  }

  closeSettingsModal() {
    this.settingsModalOpen = false
  }

  openErrorsModal() {
    this.errorsModalOpen = true
  }

  closeErrorsModal() {
    this.closeAllModals()
    this.errorsModalOpen = false
  }
}

export const uiStore = new UiStore()

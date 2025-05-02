export type Edition = "btc" | "app"

export interface Theme {
  label: string
  accent: string
  primary: string
  background: string
  videoUrl: string
  staticImage: string
}

export const themes: Record<Edition, Theme> = {
  btc: {
    label: "BTC",
    accent: "var(--color-babylon-orange)",
    primary: "var(--color-babylon-orange)",
    background: "var(--color-zinc-950)",
    videoUrl: "https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/btc-union-background-3.webm",
    staticImage: "/btc-static-video.png",
  },
  app: {
    label: "V2",
    accent: "var(--color-union)",
    primary: "var(--color-union)",
    background: "var(--color-zinc-950)",
    videoUrl: "https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/app-union-background-2.webm",
    staticImage: "/app-static-video.png",
  },
}

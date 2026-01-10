import react from "@astrojs/react"
import sitemap from "@astrojs/sitemap"
import starlight from "@astrojs/starlight"
import svelte from "@astrojs/svelte"
import starlightUtils from "@lorenzo_lewis/starlight-utils"
import tailwindcss from "@tailwindcss/vite"
import { defineConfig, passthroughImageService } from "astro/config"
import * as Fs from "node:fs/promises"
import path from "node:path"
import * as Toml from "smol-toml"
import starlightHeadingBadges from "starlight-heading-badges"
import starlightLinksValidator from "starlight-links-validator"
import starlightThemeRapide from "starlight-theme-rapide"
import Icons from "unplugin-icons/vite"
import { loadEnv, type ViteDevServer } from "vite"
import examplesToPages from "./integrations/examples-to-pages.js"
import { markdownConfiguration } from "./markdown.config.ts"

const SITE_URL = "https://docs.union.build"
const SITE_DESCRIPTION =
  "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks."

const { PORT = 4321, ENABLE_DEV_TOOLBAR = "false" } = loadEnv(
  process.env.NODE_ENV,
  process.cwd(),
  "",
)

const copyExternalDocs = () => {
  async function* walk(dir: string): AsyncGenerator<string> {
    for await (const d of await Fs.opendir(dir)) {
      const entry = path.join(dir, d.name)
      if (d.isDirectory()) {
        yield* walk(entry)
      } else if (d.isFile()) {
        yield entry
      }
    }
  }

  async function copyVoyagerDir(dir: string) {
    const voyagerDir = `../voyager/${dir}/`

    for await (const rawPath of walk(voyagerDir)) {
      const voyagerDirPath = rawPath.replace("../voyager/", "")
      await copyVoyagerFile(voyagerDirPath)
    }
  }

  async function copyVoyagerFile(voyagerDirPath: string) {
    // ignore rust files
    if (voyagerDirPath.endsWith(".rs") || voyagerDirPath.endsWith(".toml")) {
      return
    }

    console.log({ voyagerDirPath })

    const ext = path.extname(voyagerDirPath)
    if (path.basename(voyagerDirPath) === "README.md") {
      // stfu typescript
      const packageName = (Toml.parse(
        await Fs.readFile(
          path.join("../voyager", `${voyagerDirPath.replace("README.md", "Cargo.toml")}`),
          "utf8",
        ),
      ).package as Toml.TomlTable)["name"]

      const readme = await Fs.readFile(path.join("../voyager", voyagerDirPath))

      const finalPath = path.dirname(
        path.join("./src/content/docs/architecture/voyager", voyagerDirPath),
      )

      await Fs.mkdir(path.normalize(finalPath + "/.."), { recursive: true })

      await Fs.writeFile(
        finalPath + ".md",
        `---\ntitle: "${packageName}"\n---\n${readme}`,
      )
    } else if ((ext === ".json") || (ext === ".jsonc")) {
      let finalPath = path.join("./src/content/docs/architecture/voyager/", voyagerDirPath)

      console.log({ finalPath })

      await Fs.mkdir(path.dirname(finalPath), { recursive: true })

      let json = await Fs.readFile(path.join("../voyager", voyagerDirPath), "utf8")

      await Fs.writeFile(
        finalPath.replace(ext, ".mdx"),
        `
---
title: "${path.basename(finalPath)}"
---

\`\`\`${ext.replace(".", "")}
${json}
\`\`\`

`,
      )
    }
  }

  return {
    name: "copy-external-docs",
    hooks: {
      "astro:server:setup": (options: {
        server: ViteDevServer
      }) => {
        options.server.watcher
          .add("voyager/")

        options.server.watcher
          .on("add", (path) => console.log(`File ${path} has been added`))
          .on("addDir", (path) => console.log(`Dir ${path} has been added`))
          .prependListener("change", async (path) => {
            console.log(`change: ${path}`)
            if (path.startsWith("voyager/")) {
              await copyVoyagerFile(path.replace("voyager/", ""))
                .catch(e => {
                  console.log("hmr error:", e)
                })
            }
          })
      },
      "astro:config:setup": async () => {
        console.log("running copy external docs setup hook")

        await Fs.rm("./src/content/docs/architecture/voyager/", { recursive: true, force: true })

        await Fs.mkdir("./src/content/docs/architecture/voyager/doc/", { recursive: true })

        await Fs.writeFile(
          "./src/content/docs/architecture/voyager/concepts.md",
          "---\ntitle: Concepts\n---\n" + await Fs.readFile("../voyager/CONCEPTS.md", "utf8"),
        )

        await Fs.cp(
          "../voyager/doc/ibc-architecture.svg",
          "./src/content/docs/architecture/voyager/doc/ibc-architecture.svg",
        )

        await Fs.writeFile(
          "./src/content/docs/architecture/voyager/overview.md",
          "---\ntitle: Overview\n---\n" + await Fs.readFile("../voyager/README.md", "utf8"),
        )

        await copyVoyagerDir("plugins")
        await copyVoyagerDir("modules")
      },
    },
  }
}

// @ts-ignore
export default defineConfig({
  site: SITE_URL,
  output: "static",
  experimental: {
    clientPrerender: true,
    contentIntellisense: true,
  },
  trailingSlash: "ignore",
  markdown: markdownConfiguration,
  image: {
    service: passthroughImageService(),
  },
  vite: {
    resolve: {
      alias: [
        { find: "icons:svelte", replacement: "~icons" },
        { find: "icons:astro", replacement: "~icons" },
        { find: "path", replacement: "rollup-plugin-node-polyfills/polyfills/path" },
      ],
    },
    server: {
      watch: {
        cwd: "../.",
      },
    },
    plugins: [
      Icons({
        compiler: "svelte",
        autoInstall: true,
      }),
      Icons({
        compiler: "astro",
        autoInstall: true,
      }),
      tailwindcss(),
      // viteStaticCopy({
      //   targets: [
      //   ],
      // }),
    ],
    ssr: {
      noExternal: ["monaco-editor"],
    },
    optimizeDeps: {
      include: ["@xterm/xterm"],
      esbuildOptions: { target: "es2020" },
    },
  },
  server: _ => ({
    port: Number(PORT),
    /**
     * required for webcontainer
     * @see https://webcontainers.io/guides/quickstart
     */
    headers: {
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Cross-Origin-Opener-Policy": "same-origin",
    },
  }),
  redirects: { "/logo": "/union-logo.zip" },
  devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
  prefetch: { prefetchAll: true, defaultStrategy: "viewport" },
  integrations: [
    copyExternalDocs(),
    starlight({
      title: "Union",
      lastUpdated: true,
      defaultLocale: "root",
      favicon: "/favicon.svg",
      description: SITE_DESCRIPTION,
      tagline: "Connecting blockchains trustlessly",
      locales: { root: { label: "English", lang: "en" } },
      editLink: {
        baseUrl: "https://github.com/unionlabs/union/edit/main/docs/",
      },
      expressiveCode: true,
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/unionlabs" },
        { icon: "discord", label: "Discord", href: "https://discord.union.build" },
        { icon: "x.com", label: "X", href: "https://x.com/union_build" },
      ],
      logo: {
        alt: "Union Logo",
        replacesTitle: true,
        dark: "./src/assets/union-logo/union-logo-white.svg",
        light: "./src/assets/union-logo/union-logo-black.svg",
      },
      head: [
        {
          tag: "meta",
          attrs: { property: "og:image", content: "/og.png" },
        },
        {
          tag: "meta",
          attrs: { property: "twitter:image", content: "/og.png" },
        },
        {
          tag: "script",
          attrs: { src: "/scripts/anchor-targets.js" },
        },
        {
          tag: "link",
          attrs: {
            rel: "apple-touch-icon",
            href: "/pwa-192x192.png",
          },
        },
        {
          tag: "link",
          attrs: {
            rel: "mask-icon",
            href: "/favicon.svg",
            color: "#FFFFFF",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "msapplication-TileColor",
            content: "#131313",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "theme-color",
            content: "#131313",
          },
        },
      ],
      sidebar: [
        {
          label: "DOCS",
          items: [
            {
              label: "Introduction",
              link: "/",
            },
            {
              label: "Protocol",
              items: [
                {
                  label: "Overview",
                  link: "/protocol/overview",
                },
                {
                  label: "Deployments",
                  link: "/protocol/deployments",
                },
                {
                  label: "Chains",
                  autogenerate: {
                    directory: "/protocol/chains",
                  },
                },
                {
                  label: "Channels",
                  autogenerate: {
                    directory: "/protocol/channels",
                  },
                },
                {
                  label: "Connections",
                  autogenerate: {
                    directory: "/protocol/connections",
                  },
                },
              ],
            },
            {
              label: "Architecture",
              items: [
                {
                  label: "CometBLS",
                  link: "/architecture/cometbls",
                },
                {
                  label: "Galois",
                  link: "/architecture/galois",
                },
                {
                  label: "Voyager",
                  items: [
                    { label: "Overview", link: "/architecture/voyager/overview" },
                    { label: "Concepts", link: "/architecture/voyager/concepts" },
                    {
                      label: "Plugins",
                      collapsed: true,
                      autogenerate: {
                        directory: "/architecture/voyager/plugins",
                      },
                    },
                    {
                      label: "Modules",
                      collapsed: true,
                      autogenerate: {
                        directory: "/architecture/voyager/modules",
                      },
                    },
                  ],
                },
              ],
            },
            {
              label: "Connect",
              items: [
                {
                  label: "Existing Implementations",
                  link: "/connect/implementations",
                },
                {
                  label: "Integration Requirements",
                  link: "/connect/integration-requirements",
                },
                {
                  label: "New Chain",
                  items: [
                    {
                      label: "Overview",
                      link: "/connect/new-chain/overview",
                    },
                    // {
                    //   label: "EVM",
                    //   link: "/connect/new-chain/evm"
                    // },
                    // {
                    //   label: "CosmWasm",
                    //   link: "/connect/new-chain/cosmwasm"
                    // },
                    // {
                    //   label: "Move",
                    //   link: "/connect/new-chain/move"
                    // }
                  ],
                },
                {
                  label: "Apps",
                  items: [
                    {
                      label: "Asset Transfer",
                      items: [
                        {
                          label: "Introduction",
                          link: "/connect/app/asset-transfer",
                        },
                        {
                          label: "Solidity",
                          link: "/connect/app/asset-transfer/solidity",
                        },
                        {
                          label: "CosmWasm",
                          link: "/connect/app/asset-transfer/cosmwasm",
                        },
                      ],
                    },
                    // {
                    //   label: "Custom Data",
                    //   items: [
                    //     {
                    //       label: "Introduction",
                    //       link: "/connect/app/custom-data"
                    //     }
                    //   ]
                    // }
                  ],
                },
              ],
            },
            {
              label: "Concepts",
              autogenerate: {
                directory: "/concepts",
              },
            },
            {
              label: "Standards",
              autogenerate: {
                directory: "/ucs",
              },
            },
            {
              label: "Infrastructure",
              items: [
                {
                  label: "Testnet 10",
                  link: "/infrastructure/testnet-10",
                },
                {
                  label: "Mainnet",
                  link: "/infrastructure/mainnet",
                },
                {
                  label: "Node Operators",
                  collapsed: true,
                  autogenerate: {
                    directory: "/infrastructure/node-operators",
                  },
                },
              ],
            },
            {
              label: "Typescript SDK",
              items: [
                {
                  label: "Getting Started",
                  link: "/integrations/typescript",
                },
                {
                  label: "Guides (EVM)",
                  items: [
                    {
                      label: "Guide: Send Funds Holesky → Sepolia",
                      link: "/integrations/typescript/guided-tutorial/evm-holesky-sepolia",
                    },
                    {
                      label: "Example: Send Funds Holesky → Sepolia",
                      link: "/integrations/typescript/examples/evm/send-funds-holesky-to-sepolia/",
                    },
                  ],
                },
                {
                  label: "Guides (Cosmos)",
                  items: [
                    {
                      label: "Guide: Cross Chain Contract Call",
                      link: "/integrations/typescript/guided-tutorial/cosmos-call",
                    },
                    {
                      label: "Example: Cross Chain Contract Call",
                      link: "/integrations/typescript/examples/cosmos/call/",
                    },
                    {
                      label: "Guide: Send Funds Union → Sepolia",
                      link: "/integrations/typescript/guided-tutorial/cosmos-union-sepolia",
                    },
                    {
                      label: "Example: Send Funds Union → Sepolia",
                      link: "/integrations/typescript/examples/cosmos/send-funds-union-to-sepolia/",
                    },
                  ],
                },
              ],
            },
            {
              label: "Integrations",
              items: [
                {
                  label: "Getting Started",
                  link: "/integrations/getting-started",
                },
                {
                  label: "GraphQL",
                  link: "/integrations/api/graphql",
                },
              ],
            },

            {
              label: "Joining the Testnet",
              collapsed: true,
              autogenerate: {
                directory: "/joining-testnet",
              },
            },
            {
              label: "Ceremony",
              link: "/ceremony",
            },
            {
              label: "Tokenomics",
              link: "/u",
            },
            {
              label: "FAQ",
              collapsed: true,
              items: [
                {
                  label: "How to conduct multisig transfers?",
                  link: "/faq/multisig-transfers-union-btc",
                },
                {
                  label: "How to add new tokens from Union app?",
                  link: "/faq/add-tokens-to-wallet",
                },
                {
                  label: "GraphQL",
                  link: "/integrations/api/graphql",
                },
              ],
            },
          ],
        },
        {
          label: "API",
          items: [
            {
              label: "GraphQL",
              link: "/reference/graphql",
              badge: { text: "new", variant: "success" },
            },
            {
              label: "TypeScript SDK",
              autogenerate: { directory: "/reference/@unionlabs/sdk" },
              badge: { text: "evolving", variant: "caution" },
            },
            {
              label: "TypeScript SDK (EVM)",
              autogenerate: { directory: "/reference/@unionlabs/sdk-evm" },
              badge: { text: "evolving", variant: "caution" },
            },
            {
              label: "TypeScript SDK (Cosmos)",
              autogenerate: { directory: "/reference/@unionlabs/sdk-cosmos" },
              badge: { text: "evolving", variant: "caution" },
            },
          ],
        },
      ],
      plugins: [
        examplesToPages({
          baseDir: "../ts-sdk-evm/examples",
          entryPoints: [
            "../ts-sdk-evm/examples/*.ts",
          ],
          outDir: "./src/content/docs/integrations/typescript/examples/evm",
          clean: true,
        }),
        examplesToPages({
          baseDir: "../ts-sdk-cosmos/examples",
          entryPoints: [
            "../ts-sdk-cosmos/examples/*.ts",
          ],
          outDir: "./src/content/docs/integrations/typescript/examples/cosmos",
          clean: true,
        }),
        starlightThemeRapide(),
        starlightUtils({
          multiSidebar: {
            switcherStyle: "horizontalList",
          },
        }),
        starlightHeadingBadges(),
        starlightLinksValidator(),
      ],
      customCss: [
        "./src/styles/index.css",
        "./src/styles/fonts.css",
        "./src/styles/tailwind.css",
        "./node_modules/katex/dist/katex.min.css",
      ],
    }),
    sitemap(),
    svelte(),
    react({
      include: ["**/react/**"],
      experimentalReactChildren: true,
    }),
  ],
})

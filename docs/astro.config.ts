import { loadEnv } from "vite";
import vue from "@astrojs/vue";
import svelte from "@astrojs/svelte";
import sitemap from "@astrojs/sitemap";
import tailwind from "@astrojs/tailwind";
import { defineConfig } from "astro/config";
import { starlightConfig } from "./starlight.config.ts";
import { markdownConfiguration } from "./markdown.config.ts";
import starlightLinksValidatorPlugin from "starlight-links-validator";

const SITE_URL = "https://docs.union.build";

const { PORT = 4321, ENABLE_DEV_TOOLBAR = "false" } = loadEnv(
	process.env.NODE_ENV,
	process.cwd(),
	"",
);

export default defineConfig({
	site: SITE_URL,
	output: "static",
	experimental: {
		serverIslands: true,
		clientPrerender: true,
		directRenderScript: true,
		contentIntellisense: true,
	},
	trailingSlash: "ignore",
	markdown: markdownConfiguration,
	server: (_) => ({
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
	devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
	prefetch: { prefetchAll: true, defaultStrategy: "viewport" },
	redirects: { "/logo": "/union-logo.zip" },
	integrations: [
    starlightConfig,
		starlightLinksValidatorPlugin(),
		sitemap(),
		tailwind({
			applyBaseStyles: false,
			configFile: "tailwind.config.ts",
		}),
		svelte(),
		vue({ jsx: true, devtools: true }),
	],
});

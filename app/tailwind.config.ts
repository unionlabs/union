import plugin from "tailwindcss/plugin"
import type { Config } from "tailwindcss"
import tailwindAnimate from "tailwindcss-animate"
import defaultTheme from "tailwindcss/defaultTheme"
import typographyPlugin from "@tailwindcss/typography"
import tailwindScrollbarPlugin from "tailwind-scrollbar"
import aspectRatioPlugin from "@tailwindcss/aspect-ratio"
import containerQueriesPlugin from "@tailwindcss/container-queries"

export default (<Config>{
  darkMode: ["class"],
  content: ["./src/**/*.{html,js,svelte,ts}"],
  safelist: ["dark"],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px"
      }
    },
    extend: {
      fontSize: {
        md: ["1rem", { lineHeight: "1.5rem" }]
      },
      colors: {
        accent: {
          DEFAULT: "#A0ECFD",
          50: "#FAFEFF",
          100: "#F0FCFF",
          200: "#DCF8FE",
          300: "#C8F4FE",
          400: "#B4F0FD",
          500: "#A0ECFD",
          600: "#5FDFFC",
          700: "#1ED2FA",
          800: "#04ACD2",
          900: "#037791",
          950: "#025C70"
        },
        border: "hsl(var(--border) / <alpha-value>)",
        input: "hsl(var(--input) / <alpha-value>)",
        ring: "hsl(var(--ring) / <alpha-value>)",
        background: "hsl(var(--background) / <alpha-value>)",
        foreground: "hsl(var(--foreground) / <alpha-value>)",
        primary: {
          DEFAULT: "hsl(var(--primary) / <alpha-value>)",
          foreground: "hsl(var(--primary-foreground) / <alpha-value>)"
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary) / <alpha-value>)",
          foreground: "hsl(var(--secondary-foreground) / <alpha-value>)"
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive) / <alpha-value>)",
          foreground: "hsl(var(--destructive-foreground) / <alpha-value>)"
        },
        muted: {
          DEFAULT: "hsl(var(--muted) / <alpha-value>)",
          foreground: "hsl(var(--muted-foreground) / <alpha-value>)"
        },
        // accent: {
        //   DEFAULT: 'hsl(var(--accent) / <alpha-value>)',
        //   foreground: 'hsl(var(--accent-foreground) / <alpha-value>)'
        // },
        popover: {
          DEFAULT: "hsl(var(--popover) / <alpha-value>)",
          foreground: "hsl(var(--popover-foreground) / <alpha-value>)"
        },
        card: {
          DEFAULT: "hsl(var(--card) / <alpha-value>)",
          foreground: "hsl(var(--card-foreground) / <alpha-value>)"
        }
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)"
      },
      fontFamily: {
        sans: [
          "Inter var", ...defaultTheme.fontFamily.sans
        ],
        display: [
          "Inter var",
          {
            fontFeatureSettings:  '"ss01", "cv09", "ss08" "zero"',
            fontVariationSettings: '"opsz" 32'
          }
        ],
        mono: ["JetBrains Mono", ...defaultTheme.fontFamily.mono],
        jetbrains: ["JetBrains Mono", ...defaultTheme.fontFamily.mono]
      }
    }
  },
  plugins: [
    //
    tailwindAnimate,
    typographyPlugin,
    aspectRatioPlugin,
    containerQueriesPlugin,
    tailwindScrollbarPlugin,
    plugin(({ addVariant, addUtilities, matchUtilities, theme }) => {
      matchUtilities(
        { "animation-delay": value => ({ "animation-delay": value }) },
        { values: theme("transitionDelay") }
      )
      addVariant("optional", "&:optional")
      addVariant("hocus", ["&:hover", "&:focus"])
      addUtilities({
        ".content-auto": { "content-visibility": "auto" },
        ".content-hidden": { "content-visibility": "hidden" },
        ".content-visible": { "content-visibility": "visible" }
      })
      addVariant("inverted-colors", "@media (inverted-colors: inverted)")
    })
  ]
})

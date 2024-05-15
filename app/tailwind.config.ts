import plugin from "tailwindcss/plugin"
import type { Config } from "tailwindcss"
import formsPlugin from "@tailwindcss/forms"
import tailwindAnimate from "tailwindcss-animate"
import defaultTheme from "tailwindcss/defaultTheme"
import typographyPlugin from "@tailwindcss/typography"
import tailwindScrollbarPlugin from "tailwind-scrollbar"
import aspectRatioPlugin from "@tailwindcss/aspect-ratio"
import containerQueriesPlugin from "@tailwindcss/container-queries"

export default {
  darkMode: ["class"],
  content: ["./src/**/*.{html,js,svelte,ts}"],
  safelist: ["dark"],
  theme: {
    screens: _units => ({
      xs: "475px",
      ...defaultTheme.screens,
      "2xl": "1400px",
      tall: { raw: "(min-height: 800px)" }
    }),
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
      cursor: {
        fancy: "url(hand.cur), pointer"
      },
      colors: {
        // accent: {
        //   DEFAULT: "#A0ECFD",
        //   50: "#FAFEFF",
        //   100: "#F0FCFF",
        //   200: "#DCF8FE",
        //   300: "#C8F4FE",
        //   400: "#B4F0FD",
        //   500: "#A0ECFD",
        //   600: "#5FDFFC",
        //   700: "#1ED2FA",
        //   800: "#04ACD2",
        //   900: "#037791",
        //   950: "#025C70",
        //   foreground: "hsl(var(--accent-foreground) / <alpha-value>)"
        // },
        border: "hsl(var(--border) / <alpha-value>)",
        input: "hsl(var(--input) / <alpha-value>)",
        ring: "hsl(var(--ring))",
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
        accent: {
          DEFAULT: "hsl(var(--accent) / <alpha-value>)",
          foreground: "hsl(var(--accent-foreground) / <alpha-value>)"
        },
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
        sans: ["Inter var", ...defaultTheme.fontFamily.sans],
        display: [
          "Inter var",
          {
            fontFeatureSettings: '"ss01", "cv09", "ss08" "zero"',
            fontVariationSettings: '"opsz" 32'
          }
        ],
        mono: ["JetBrains Mono", ...defaultTheme.fontFamily.mono],
        jetbrains: ["JetBrains Mono", ...defaultTheme.fontFamily.mono]
      },
      animation: {
        wiggle: "wiggle 1s ease-in-out infinite",
        "text-gradient": "text-gradient 1.5s linear infinite",
        "background-shine": "background-shine 2s linear infinite",
        "pulse-slow": "pulse 6s infinite cubic-bezier(0.4, 0, 0.6, 1)",
        "border-width": "border-width 3s infinite alternate"
      },
      animationDuration: { "2s": "2s" },
      keyframes: {
        "text-gradient": {
          to: { backgroundPosition: "200% center" }
        },
        "background-shine": {
          from: { backgroundPosition: "0 0" },
          to: { backgroundPosition: "-200% 0" }
        },
        "border-width": {
          from: { width: "10px", opacity: "0" },
          to: { width: "100px", opacity: "1" }
        },
        wiggle: {
          "0%, 100%": { transform: "rotate(-3deg)" },
          "50%": { transform: "rotate(3deg)" }
        }
      },
      transitionDelay: { "2000": "2000ms" },
      transitionProperty: { height: "height", spacing: "margin, padding" },
      transitionTimingFunction: {
        "in-expo": "cubic-bezier(0.95, 0.05, 0.795, 0.035)",
        "out-expo": "cubic-bezier(0.19, 1, 0.22, 1)"
      }
    }
  },
  plugins: [
    tailwindAnimate,
    aspectRatioPlugin,
    typographyPlugin(),
    containerQueriesPlugin,
    tailwindScrollbarPlugin,
    tailwindAnimationDelay(),
    formsPlugin({ strategy: "class" }),
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
} satisfies Config

function tailwindAnimationDelay() {
  return plugin(({ addUtilities, theme, e }) => {
    const defaultValues = {
      none: "0s",
      75: "75ms",
      100: "100ms",
      150: "150ms",
      200: "200ms",
      300: "300ms",
      400: "400ms",
      500: "500ms",
      600: "600ms",
      700: "700ms",
      800: "800ms",
      900: "900ms",
      1000: "1000ms",
      1100: "1100ms",
      1200: "1200ms",
      1300: "1300ms",
      1400: "1400ms",
      1500: "1500ms",
      2000: "2000ms",
      3000: "3000ms",
      4000: "4000ms",
      5000: "5000ms",
      6000: "6000ms",
      7000: "7000ms",
      8000: "8000ms",
      9000: "9000ms"
    }
    const userValues = theme("animationDelay")
    const values = { ...defaultValues, ...userValues }
    const utilities = Object.entries(values).map(([key, value]) => ({
      [`.${e(`animation-delay-${key}`)}`]: { animationDelay: `${value}` }
    }))
    addUtilities(utilities)
  })
}

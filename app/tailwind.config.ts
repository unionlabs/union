import plugin from 'tailwindcss/plugin'
import type { Config } from 'tailwindcss'
import tailwindAnimate from 'tailwindcss-animate'
import defaultTheme from 'tailwindcss/defaultTheme'
import typographyPlugin from '@tailwindcss/typography'
import tailwindScrollbarPlugin from 'tailwind-scrollbar'
import aspectRatioPlugin from '@tailwindcss/aspect-ratio'
import containerQueriesPlugin from '@tailwindcss/container-queries'

export default <Config>{
  darkMode: 'class',
  future: { hoverOnlyWhenSupported: true },
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {}
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
        { 'animation-delay': value => ({ 'animation-delay': value }) },
        { values: theme('transitionDelay') }
      )
      addVariant('optional', '&:optional')
      addVariant('hocus', ['&:hover', '&:focus'])
      addUtilities({
        '.content-auto': { 'content-visibility': 'auto' },
        '.content-hidden': { 'content-visibility': 'hidden' },
        '.content-visible': { 'content-visibility': 'visible' }
      })
      addVariant('inverted-colors', '@media (inverted-colors: inverted)')
    })
  ]
}

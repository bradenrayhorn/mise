import plugin from 'tailwindcss/plugin';

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'selector',

  plugins: [
    plugin(function ({ addVariant }) {
      addVariant('stuck', '&[data-stuck]');
    }),

    plugin(function ({ matchUtilities, theme }) {
      matchUtilities(
        {
          pbsafe: (value) => ({
            paddingBottom: `calc(env(safe-area-inset-bottom, 0px) + ${value})`,
          }),
          ptsafe: (value) => ({
            paddingTop: `calc(env(safe-area-inset-top, 0px) + ${value})`,
          }),
        },
        { values: theme('spacing') },
      );
    }),
  ],

  theme: {
    colors: {
      food: {
        1: 'var(--color-food-1)',
        2: 'var(--color-food-2)',
        3: 'var(--color-food-3)',
        4: 'var(--color-food-4)',
        5: 'var(--color-food-5)',
        6: 'var(--color-food-6)',
        7: 'var(--color-food-7)',
        8: 'var(--color-food-8)',
        9: 'var(--color-food-9)',
        10: 'var(--color-food-10)',
      },

      alpha: {
        50: 'var(--color-alpha-50)',
        100: 'var(--color-alpha-100)',
        200: 'var(--color-alpha-200)',
        300: 'var(--color-alpha-300)',
        400: 'var(--color-alpha-400)',
        500: 'var(--color-alpha-500)',
        600: 'var(--color-alpha-600)',
        700: 'var(--color-alpha-700)',
        800: 'var(--color-alpha-800)',
        900: 'var(--color-alpha-900)',
      },

      base: {
        500: 'var(--color-base-500)',
        600: 'var(--color-base-600)',

        skeleton: 'var(--color-base-skeleton)',

        primaryHover: 'var(--color-base-primaryHover)',

        backdrop: 'var(--color-base-backdrop)',
      },

      divider: {
        default: 'var(--color-divider-default)',
      },

      fg: {
        highlight: 'var(--color-fg-highlight)',
        default: 'var(--color-fg-default)',
        muted: 'var(--color-fg-muted)',
      },

      link: {
        primary: 'var(--color-link-primary)',
      },

      button: {
        fgPrimary: 'var(--color-button-fgPrimary)',
        bgPrimary: 'var(--color-button-bgPrimary)',
        bgPrimaryHover: 'var(--color-button-bgPrimaryHover)',
        bgPrimaryActive: 'var(--color-button-bgPrimaryActive)',
      },

      tag: {
        bg: 'var(--color-tag-bg)',
        fg: 'var(--color-tag-fg)',
        divider: 'var(--color-tag-divider)',
      },

      inputBorder: {
        default: 'var(--color-inputBorder-default)',
        active: 'var(--color-inputBorder-active)',
      },
    },
    extend: {
      boxShadow: {
        outline: '0 0 0 1px',
      },

      data: {
        active: 'active="true"',
      },
    },
  },
};

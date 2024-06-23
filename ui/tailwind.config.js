const colors = require('tailwindcss/colors');

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'selector',
  theme: {
    extend: {
      fontFamily: {
        sans: ['NotoSans', 'ui-sans-serif'],
        serif: ['NotoSerif', 'ui-serif'],
      },
      colors: {
        neutral: colors.gray,
        primary: {
          50: '#F2F7F5',
          100: '#E6F0EB',
          200: '#C9DED4',
          300: '#A6C9B8',
          400: '#81B29A',
          500: '#6DA68B',
          600: '#5D987B',
          700: '#4F8269',
          800: '#446F5A',
          900: '#2E4C3E',
          950: '#21362C',
        },
      },
    },
  },
};

import { createThemes } from 'tw-colors';

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
      boxShadow: {
        outline: '0 0 0 1px',
      },
    },
  },
  plugins: [
    createThemes(
      {
        light: {
          //
          text: {
            100: colors.zinc['950'],
            200: colors.zinc['800'],
            300: colors.zinc['700'],

            primary: colors.zinc['100'],
          },
          //
          // Color page and object backgrounds. Higher numbers means content is at a higher
          // elevation.
          base: {
            50: '#FAFAFA', // darker
            100: '#F9FAFA', // base color of page
            200: '#FCFDFD',
            300: '#FCFDFD',
            400: '#FCFDFD',
          },
          //
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
          //
          neutral: {
            50: '#F1F2F3',
            100: '#E0E2E5',
            200: '#C2C5CC',
            300: '#A6ABB5',
            400: '#888E9B',
            500: '#6B7280',
            600: '#5D636F',
            700: '#4F545E',
            800: '#3F434B',
            900: '#31343A',
            950: '#2A2D32',
          },
          //
          food: {
            1: '#1C3144',
            2: '#2E5266',
            3: '#8A7090',
            4: '#FE654F',
            5: '#00798C',
            6: '#595A4A',
            7: '#F2A359',
            8: '#DE3C4B',
            9: '#37371F',
            10: '#957FEF',
          },
        },
        dark: {
          //
          text: {
            100: colors.zinc['100'],
            200: colors.zinc['200'],
            300: colors.zinc['300'],
          },
          //
          // Color page and object backgrounds. Higher numbers means content is at a higher
          // elevation.
          base: {
            50: '#25282C', // darker
            100: '#2A2D32', // base color of page
            200: '#31343A',
            300: '#3F434B',
            400: '#4F545E',
          },
          //
          primary: {
            950: '#F2F7F5',
            900: '#E6F0EB',
            800: '#C9DED4',
            700: '#A6C9B8',
            600: '#81B29A',
            500: '#6DA68B',
            400: '#5D987B',
            300: '#4F8269',
            200: '#446F5A',
            100: '#2E4C3E',
            50: '#21362C',
          },
          //
          neutral: {
            950: '#F1F2F3',
            900: '#E0E2E5',
            800: '#C2C5CC',
            700: '#A6ABB5',
            600: '#888E9B',
            500: '#6B7280',
            400: '#5D636F',
            300: '#4F545E',
            200: '#3F434B',
            100: '#31343A',
            50: '#2A2D32',
          },
        },
      },
      { strict: true, defaultTheme: 'light' },
    ),
  ],
};

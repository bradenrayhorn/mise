import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';
import Icons from 'unplugin-icons/vite';
import license from 'rollup-plugin-license';
import path from 'path';

export default defineConfig({
  plugins: [
    sveltekit(),
    Icons({ compiler: 'svelte' }),
    {
      ...license({
        sourcemap: true,
        thirdParty: {
          multipleVersions: true,
          allow: {
            test: '(MIT OR ISC)',
            failOnViolation: true,
            failOnUnlicensed: true,
          },
          output: {
            file: path.join(__dirname, 'static', 'licenses', 'ui-licenses.txt'),
          },
        },
      }),
      enforce: 'post',
      apply: () => {
        return !!process.env.GENERATE_LICENSES;
      },
    },
  ],
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
  },
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:3000',
      '/auth': 'http://127.0.0.1:3000',
      '^/$': 'http://127.0.0.1:3000',
    },
  },
});

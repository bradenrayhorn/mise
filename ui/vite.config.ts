import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';
import Icons from 'unplugin-icons/vite';

export default defineConfig({
  plugins: [sveltekit(), Icons({ compiler: 'svelte' })],
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

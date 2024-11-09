<script lang="ts">
  import '../app.postcss';
  import { initTheme } from '$lib/theme-switch';
  import LightSwitch from '$lib/components/LightSwitch.svelte';
  import { browser, dev } from '$app/environment';
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
  import type { MaybeError } from '$lib/types/error';
  import LoginIfUnauthenticated from '$lib/components/LoginIfUnauthenticated.svelte';
  import type { Snippet } from 'svelte';

  type Props = {
    children?: Snippet;
  };

  let { children }: Props = $props();

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        enabled: browser,
        staleTime: 60 * 10 * 1000,
        retry: (failureCount, error) => {
          const maybeError = error as MaybeError;
          if (maybeError?.status === 401) {
            return false;
          }

          return failureCount < 3;
        },
      },
    },
  });

  initTheme();
</script>

<QueryClientProvider client={queryClient}>
  <LoginIfUnauthenticated>
    {@render children?.()}
  </LoginIfUnauthenticated>
</QueryClientProvider>

{#if dev}
  <div class="hidden md:block fixed bottom-2 right-2 opacity-0 hover:opacity-100">
    <LightSwitch />
  </div>
{/if}

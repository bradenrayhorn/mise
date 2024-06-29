<script>
  import '../app.postcss';
  import { initTheme } from '$lib/theme-switch';
  import LightSwitch from '$lib/components/LightSwitch.svelte';
  import { dev } from '$app/environment';

  initTheme();
</script>

<slot />

{#if dev}
  <div class="hidden md:block fixed bottom-2 right-2 opacity-0 hover:opacity-100">
    <LightSwitch />
  </div>
{/if}

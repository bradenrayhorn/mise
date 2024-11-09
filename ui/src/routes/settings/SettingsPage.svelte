<script lang="ts">
  import IconBack from '~icons/mdi/chevron-left';
  import IconAbout from '~icons/mdi/about-variant';
  import IconLight from '~icons/mdi/weather-sunny';
  import IconDark from '~icons/mdi/moon-and-stars';
  import { setTheme, isThemeDark } from '$lib/theme-switch';

  type Props = {
    backURL: string;
  };
  const { backURL }: Props = $props();

  let isDark = $state(isThemeDark());
</script>

<div class="w-full h-full">
  <div class="flex flex-col max-w-md mx-auto px-4 gap-8">
    <div class="ptsafe-6 pb-6">
      <h1 class="font-serif font-bold text-3xl mb-4">Settings</h1>

      <a href={backURL} class="flex items-center">
        <IconBack />
        <span class="text-sm">Back to recipes</span>
      </a>
    </div>

    <button
      class="flex items-center gap-2"
      onclick={() => {
        setTheme(!isDark);
        isDark = !isDark;
      }}
    >
      {#if isDark}
        <IconLight />
      {:else}
        <IconDark />
      {/if}
      <span class="text-sm">
        {#if isDark}
          Switch to light mode
        {:else}
          Switch to dark mode
        {/if}
      </span>
    </button>

    <a href="/about" class="flex items-center gap-2">
      <IconAbout />
      <span class="text-sm">About mise</span>
    </a>
  </div>
</div>

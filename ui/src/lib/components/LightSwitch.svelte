<script lang="ts">
  import IconLight from '~icons/mdi/weather-sunny';
  import IconDark from '~icons/mdi/moon-and-stars';
  import { setTheme, isThemeDark } from '$lib/theme-switch';

  let isDark = $state(isThemeDark());
</script>

<button
  class="w-8 h-8 bg-base-500 shadow rounded-full flex items-center justify-center"
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
</button>

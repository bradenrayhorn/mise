<script lang="ts">
  import IconOptions from '~icons/mdi/dots-horizontal-circle-outline';
  import { createDropdownMenu, melt } from '@melt-ui/svelte';

  const {
    elements: { trigger, menu, item, separator },
    states: { open },
  } = createDropdownMenu({
    preventScroll: false,
  });
</script>

<button
  use:melt={$trigger}
  class="text-xl text-fg-muted"
  data-active={$open}
  aria-label="More options"
>
  <IconOptions aria-hidden="true" />
</button>

{#if open}
  <div class="z-10 flex flex-col shadow rounded bg-base-600 py-2 text-sm" use:melt={$menu}>
    <a
      href="/recipes/new"
      class="pl-3 pr-8 data-[highlighted]:bg-base-primaryHover py-1 text-left"
      use:melt={$item}
    >
      Add recipe
    </a>

    <div class="h-px shrink-0 bg-divider-default my-2" use:melt={$separator}></div>

    <a
      href="/settings"
      class="pl-3 pr-8 data-[highlighted]:bg-base-primaryHover py-1 text-left"
      use:melt={$item}
    >
      Settings
    </a>
  </div>
{/if}

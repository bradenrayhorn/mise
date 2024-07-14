<script lang="ts">
  import DeleteIcon from '~icons/mdi/close';
  export let canDelete: boolean;
</script>

<div
  class="rounded-full bg-tag-bg text-tag-fg py-1 font-semibold text-sm w-fit flex"
  class:px-4={!canDelete}
  class:pl-4={canDelete}
>
  <slot />
  {#if canDelete}
    <button class="ml-2 px-2 border-l border-tag-divider" aria-label="Delete tag" on:click>
      <DeleteIcon />
    </button>
  {/if}
</div>

<script lang="ts">
  import DeleteIcon from '~icons/mdi/close';
  export let canDelete: boolean;
</script>

<div
  class="rounded-full bg-primary-100 text-primary-700 py-1 font-semibold text-sm w-fit flex"
  class:px-4={!canDelete}
  class:pl-4={canDelete}
>
  <slot />
  {#if canDelete}
    <button class="ml-2 px-2 border-l border-primary-600" on:click>
      <DeleteIcon />
    </button>
  {/if}
</div>

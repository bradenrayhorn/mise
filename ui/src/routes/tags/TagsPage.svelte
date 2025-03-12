<script lang="ts">
  import type { Tag } from '$lib/types/tag';

  const { tags }: { tags: Tag[] } = $props();
</script>

Tags

{#each tags as tag (tag.id)}
  <div>{tag.name}</div></div>
{/each}

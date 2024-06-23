<script lang="ts">
  import Recipe from '$lib/components/recipes/Recipe.svelte';
  import type { PageData } from './$types';

  export let data: PageData;
</script>

<div class="w-full h-full">
  <a href={`/recipes/${data.id}/edit`}>Edit</a>
  <Recipe recipe={data.recipe} />
</div>

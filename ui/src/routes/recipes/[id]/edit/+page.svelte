<script lang="ts">
  import { superForm } from 'sveltekit-superforms';
  import type { PageData } from './$types';
  import { zodClient } from 'sveltekit-superforms/adapters';
  import TitleField from '$lib/components/recipes/form/TitleField.svelte';
  import NotesField from '$lib/components/recipes/form/NotesField.svelte';
  import IngredientsField from '$lib/components/recipes/form/IngredientsField.svelte';
  import { schema } from '$lib/components/recipes/form/schema';
  import ImageField from '$lib/components/recipes/form/ImageField.svelte';
  import TagsField from '$lib/components/recipes/form/TagsField.svelte';
  import { updateRecipe } from '$lib/api/action/recipe';
  import { handleSuperformError, superformGoto } from '$lib/error-to-superform';
  import { page } from '$app/stores';

  export let data: PageData;
  const superform = superForm(data.form, {
    SPA: true,
    dataType: 'json',
    validators: zodClient(schema),
    resetForm: false,
    onUpdate: async function ({ form }) {
      if (!form.valid) {
        return;
      }

      try {
        await updateRecipe({
          fetch,
          url: $page.url,
          id: data.id,
          hash: data.hash,
          currentRecipe: data.recipe,
          recipe: {
            title: form.data.title,
            image: form.data.image,
            notes: form.data.notes,
            ingredient_blocks: form.data.ingredient_blocks,
            instruction_blocks: [],
            tags: form.data.tags,
          },
        });
      } catch (error: any) {
        await handleSuperformError(form, error, superformGoto);
      }
    },
  });

  const { form, errors, enhance } = superform;
</script>

<form method="POST" use:enhance>
  <div class="flex justify-between mb-16">
    <h1 class="font-bold text-3xl font-serif">edit recipe</h1>

    <button type="submit">Save</button>
  </div>

  <div class="flex flex-col md:flex-row gap-8 px-2">
    <div class="flex-1 flex flex-col">
      <h2>Attributes</h2>

      <ImageField {superform} />

      <TitleField bind:value={$form.title} errors={$errors.title} />

      <NotesField bind:value={$form.notes} errors={$errors.notes} />

      <TagsField {superform} promisedTags={data.tags} />
    </div>

    <div class="flex-1">
      <h2>Ingredients</h2>

      <IngredientsField {superform} />
    </div>

    <div class="flex-1">
      <h2>Instructions</h2>
    </div>
  </div>
</form>

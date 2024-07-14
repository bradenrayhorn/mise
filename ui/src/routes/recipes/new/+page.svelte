<script lang="ts">
  import { superForm } from 'sveltekit-superforms';
  import type { PageData } from './$types';
  import { zodClient } from 'sveltekit-superforms/adapters';
  import TitleField from '$lib/components/recipes/form/TitleField.svelte';
  import NotesField from '$lib/components/recipes/form/NotesField.svelte';
  import IngredientsField from '$lib/components/recipes/form/IngredientsField.svelte';
  import { schema } from '$lib/components/recipes/form/schema';
  import ImageField from '$lib/components/recipes/form/ImageField.svelte';
  import { createRecipe } from '$lib/api/action/recipe';
  import TagsField from '$lib/components/recipes/form/TagsField.svelte';
  import { handleSuperformError, superformGoto } from '$lib/error-to-superform';
  import { page } from '$app/stores';
  import InstructionsField from '$lib/components/recipes/form/InstructionsField.svelte';
  import { goto } from '$app/navigation';
  import Button from '$lib/components/Button.svelte';
  import CloseIconButton from '$lib/components/CloseIconButton.svelte';

  export let data: PageData;
  const superform = superForm(data.form, {
    SPA: true,
    dataType: 'json',
    validators: zodClient(schema),
    onUpdate: async function ({ form }) {
      if (!form.valid) {
        return;
      }

      try {
        await createRecipe({
          fetch,
          url: $page.url,
          recipe: {
            title: form.data.title,
            image: form.data.image,
            notes: form.data.notes,
            ingredient_blocks: form.data.ingredient_blocks,
            instruction_blocks: form.data.instruction_blocks,
            tags: form.data.tags,
          },
        });
      } catch (error) {
        await handleSuperformError(form, error, superformGoto);
      }

      goto('/recipes');
    },
  });

  const { enhance, submitting } = superform;
</script>

<div class="absolute top-1 left-1 z-10 flex items-center">
  <CloseIconButton href={data.backURL} />
</div>

<form method="POST" use:enhance class="pb-8">
  <div class="flex justify-between items-baseline mb-8 px-4 md:px-8 lg:px-12 pt-12">
    <h1 class="font-bold text-3xl font-serif">Add Recipe</h1>

    <Button type="submit" class="btn-solid btn-primary" isLoading={$submitting}>Save</Button>
  </div>

  <div class="flex flex-col md:flex-row gap-8 px-4 md:px-8 lg:px-12">
    <div class="flex-1 flex flex-col gap-6">
      <TitleField {superform} />

      <ImageField {superform} />

      <NotesField {superform} />

      <TagsField {superform} promisedTags={data.tags} />
    </div>

    <div class="flex-1">
      <h2 class="text-xl font-bold font-serif mb-4 md:mb-6">Ingredients</h2>

      <IngredientsField {superform} />
    </div>

    <div class="flex-1">
      <h2 class="text-xl font-bold font-serif mb-4 md:mb-6">Instructions</h2>

      <InstructionsField {superform} />
    </div>
  </div>
</form>

<script lang="ts">
  import { superForm } from 'sveltekit-superforms';
  import type { PageData } from './$types';
  import { zodClient } from 'sveltekit-superforms/adapters';
  import TitleField from '$lib/components/recipes/form/TitleField.svelte';
  import NotesField from '$lib/components/recipes/form/NotesField.svelte';
  import IngredientsField from '$lib/components/recipes/form/IngredientsField.svelte';
  import { schema } from '$lib/components/recipes/form/schema';
  import ImageField from '$lib/components/recipes/form/ImageField.svelte';
  import { uploadImage } from '$lib/api/requests/image';
  import TagsField from '$lib/components/recipes/form/TagsField.svelte';

  export let data: PageData;
  const superform = superForm(data.form, {
    SPA: true,
    dataType: 'json',
    validators: zodClient(schema),
    onUpdate: async function ({ form }) {
      console.log({ form });
      if (form.valid) {
        await fetch('/api/v1/recipes', {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
          },
          body: JSON.stringify({
            title: form.data.title,
            image_id: form.data.image ? await uploadImage(form.data.image) : undefined,
            notes: form.data.notes.trim().length > 0 ? form.data.notes.trim() : null,
            ingredients: form.data.ingredient_blocks
              .map((b) => ({ ...b, ingredients: b.ingredients.filter((i) => i.trim().length > 0) }))
              .filter((b) => b.ingredients.length > 0),
            instructions: [],
            tag_ids: form.data.tags.map((t) => t.id),
          }),
        });
      }
    },
  });

  const { form, errors, enhance } = superform;
</script>

<form method="POST" use:enhance>
  <div class="flex justify-between mb-16">
    <h1 class="font-bold text-3xl font-serif">new recipe</h1>

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

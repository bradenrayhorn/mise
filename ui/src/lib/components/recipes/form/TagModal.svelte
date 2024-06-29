<script lang="ts">
  import IconClose from '~icons/mdi/close-thick';
  import { createDialog, melt, type AnyMeltElement } from '@melt-ui/svelte';
  import { defaults, superForm } from 'sveltekit-superforms';
  import { zod } from 'sveltekit-superforms/adapters';
  import { schema } from './tag-schema';
  import { fade, slide } from 'svelte/transition';

  export let element: AnyMeltElement;

  const {
    elements: { trigger, portalled, overlay, content, title, close },
    states: { open },
  } = createDialog();

  const { form, errors, enhance } = superForm(defaults(zod(schema)), {
    SPA: true,
    validators: zod(schema),
    onUpdate: async function ({ form }) {
      console.log({ form });
      if (form.valid) {
        await fetch(`/api/v1/tags`, {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
          },
          body: JSON.stringify({
            name: form.data.name,
          }),
        });
      }
    },
  });
</script>

<button
  use:melt={$element}
  use:melt={$trigger}
  class="pl-3 w-full text-left data-[highlighted]:bg-primary-100 text-text-200">Create Tag</button
>

{#if $open}
  <div use:melt={$portalled}>
    <div
      use:melt={$overlay}
      class="fixed z-40 bg-black/50 top-0 bottom-0 right-0 left-0"
      transition:fade={{ duration: 100 }}
    />
    <div
      use:melt={$content}
      class="fixed z-50 bottom-0 left-0 right-0 md:bottom-1/2 md:left-1/2 md:-translate-x-1/2 bg-base-100 rounded-t-xl md:rounded-lg md:max-w-96 flex flex-col"
      transition:fade={{ duration: 100 }}
    >
      <form method="POST" use:enhance>
        <div class="flex items-center p-4 border-b-neutral-100 border-b mb-4 shrink-0">
          <div class="flex-1 flex items-center">
            <button use:melt={$close} class="rounded-full bg-neutral-100 text-neutral-700 p-1"
              ><IconClose /></button
            >
          </div>
          <h2 use:melt={$title} class="font-semibold">Create Tag</h2>
          <div class="flex-1" />
        </div>

        <div class="px-4 mb-6">
          <label>
            Name
            <input
              class="input"
              aria-invalid={$errors.name ? 'true' : undefined}
              bind:value={$form.name}
            />
          </label>
          {#if $errors.name}<span class="invalid">{$errors.name}</span>{/if}
        </div>

        <div class="shrink-0 p-4 flex gap-2">
          <button
            use:melt={$close}
            class="border-neutral-800 text-neutral-800 border-2 rounded w-full py-2 font-semibold"
            >Cancel</button
          >
          <button
            type="submit"
            class="bg-primary-800 text-neutral-50 dark:bg-primary-200 dark:text-neutral-950 rounded w-full py-2 font-semibold"
            >Save</button
          >
        </div>
      </form>
    </div>
  </div>
{/if}

<script lang="ts">
  import IconClose from '~icons/mdi/close';
  import { createDialog, melt, type AnyMeltElement } from '@melt-ui/svelte';
  import { defaults, superForm } from 'sveltekit-superforms';
  import { zod } from 'sveltekit-superforms/adapters';
  import { schema } from './tag-schema';
  import { fade } from 'svelte/transition';
  import Button from '$lib/components/Button.svelte';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { queryKeys } from '$lib/api/query-keys';

  export let element: AnyMeltElement;

  const client = useQueryClient();

  const {
    elements: { trigger, portalled, overlay, content, title, close },
    states: { open },
  } = createDialog({ closeOnOutsideClick: false });

  const { form, errors, enhance, submitting } = superForm(defaults(zod(schema)), {
    SPA: true,
    validators: zod(schema),
    onUpdate: async function ({ form }) {
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

        await client.invalidateQueries({ queryKey: [queryKeys.tag.list] });

        $open = false;
      }
    },
  });
</script>

<button
  use:melt={$element}
  use:melt={$trigger}
  class="pl-3 w-full text-left data-[highlighted]:bg-base-primaryHover">Create Tag</button
>

{#if $open}
  <div use:melt={$portalled}>
    <div
      use:melt={$overlay}
      class="fixed z-40 bg-base-backdrop top-0 bottom-0 right-0 left-0"
      aria-hidden="true"
      on:click|stopPropagation={() => {
        $open = false;
      }}
      transition:fade={{ duration: 100 }}
    />
    <div
      use:melt={$content}
      class="fixed z-50 bottom-0 left-0 right-0 md:bottom-1/2 md:left-1/2 md:-translate-x-1/2 bg-base-500 rounded-t-xl md:rounded-lg md:max-w-96 flex flex-col"
      transition:fade={{ duration: 100 }}
    >
      <form method="POST" use:enhance>
        <div class="flex items-center p-4 border-b-divider-default border-b mb-4 shrink-0">
          <div class="flex-1 flex items-center">
            <button use:melt={$close} class="rounded-full text-fg-muted p-1 text-lg"
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
          <button use:melt={$close} class="btn-solid btn-gray btn-sm grow" disabled={$submitting}
            >Cancel</button
          >
          <Button type="submit" class="btn-solid btn-primary btn-sm grow" isLoading={$submitting}
            >Save</Button
          >
        </div>
      </form>
    </div>
  </div>
{/if}

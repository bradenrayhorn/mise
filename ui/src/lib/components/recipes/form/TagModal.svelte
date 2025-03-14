<script lang="ts">
  import IconClose from '~icons/mdi/close';
  import { defaults, superForm } from 'sveltekit-superforms';
  import { zod } from 'sveltekit-superforms/adapters';
  import { schema } from './tag-schema';
  import { fade } from 'svelte/transition';
  import Button from '$lib/components/Button.svelte';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { queryKeys } from '$lib/api/query-keys';
  import * as dialog from '@zag-js/dialog';
  import { portal, normalizeProps, useMachine } from '@zag-js/svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  type Props = HTMLAttributes<HTMLElement>;

  const rest: Props = $props();

  const client = useQueryClient();

  const id = $props.id();
  const service = useMachine(dialog.machine, { id });
  const api = $derived(dialog.connect(service, normalizeProps));

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

        api.setOpen(false);
      }
    },
  });
</script>

<button
  {...rest}
  {...api.getTriggerProps()}
  class="pl-3 w-full text-left data-[highlighted]:bg-base-primaryHover">Create Tag</button
>

{#if api.open}
  <div
    use:portal
    {...api.getBackdropProps()}
    class="fixed z-40 bg-base-backdrop top-0 bottom-0 right-0 left-0"
    aria-hidden="true"
    transition:fade={{ duration: 100 }}
  ></div>
  <div
    use:portal
    {...api.getPositionerProps()}
    class="fixed z-50 bottom-0 left-0 right-0 md:bottom-1/2 md:left-1/2 md:-translate-x-1/2 bg-base-500 rounded-t-xl md:rounded-lg md:max-w-96 flex flex-col"
    transition:fade={{ duration: 100 }}
  >
    <form method="POST" use:enhance {...api.getContentProps()}>
      <div class="flex items-center p-4 border-b-divider-default border-b mb-4 shrink-0">
        <div class="flex-1 flex items-center">
          <button {...api.getCloseTriggerProps()} class="rounded-full text-fg-muted p-1 text-lg"
            ><IconClose /></button
          >
        </div>
        <h2 {...api.getTitleProps()} class="font-semibold">Create Tag</h2>
        <div class="flex-1"></div>
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
          {...api.getCloseTriggerProps()}
          class="btn-solid btn-gray btn-sm grow"
          disabled={$submitting}>Cancel</button
        >
        <Button type="submit" class="btn-solid btn-primary btn-sm grow" isLoading={$submitting}
          >Save</Button
        >
      </div>
    </form>
  </div>
{/if}

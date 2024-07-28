import { getContext, setContext } from 'svelte';
import { writable, type Writable } from 'svelte/store';

const contextKey = 'mise_navigating_to';

type NavigatingContext = Writable<{
  to: string | undefined;
}>;

export function useNavigating(): NavigatingContext {
  return getContext<NavigatingContext>(contextKey);
}

export function initNavigatingContext() {
  setContext<NavigatingContext>(contextKey, writable({ to: undefined }));
}

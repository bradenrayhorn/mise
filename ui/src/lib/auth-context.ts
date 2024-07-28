import { getContext, setContext } from 'svelte';
import { writable, type Writable } from 'svelte/store';

const contextKey = 'mise_auth';

export type AuthContext = Writable<{
  unauthenticated: boolean;
}>;

export function useAuth(): AuthContext {
  return getContext<AuthContext>(contextKey);
}

export function initAuthContext() {
  setContext<AuthContext>(contextKey, writable({ unauthenticated: false }));
}

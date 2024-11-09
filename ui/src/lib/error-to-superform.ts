import { setMessage, type SuperValidated } from 'sveltekit-superforms';
import { auth } from './auth.svelte';

export async function handleSuperformError<T extends Record<string, unknown>>(
  form: SuperValidated<T>,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  error: any,
) {
  console.error('form error: ', error);

  if (error?.status === 401) {
    auth.unauthenticated = true;
    return {};
  }

  setMessage(form, error?.message ?? 'Unknown error.');
}

import { setMessage, type SuperValidated } from 'sveltekit-superforms';
import type { AuthContext } from './auth-context';

export async function handleSuperformError<T extends Record<string, unknown>>(
  form: SuperValidated<T>,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  error: any,
  auth: AuthContext,
) {
  console.error('form error: ', error);

  if (error?.status === 401) {
    auth.update(() => ({ unauthenticated: true }));
    return {};
  }

  setMessage(form, error?.message ?? 'Unknown error.');
}

import type { APICall } from '../fetch';
import { handleAPIError } from '../handle-error';

export const getSelf = async ({ fetch: _fetch }: APICall): Promise<string> => {
  const res = await _fetch('/api/v1/auth/me');

  if (!res.ok) {
    await handleAPIError(res);
  }

  return 'self';
};

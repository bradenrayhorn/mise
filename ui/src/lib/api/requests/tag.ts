import { error } from '@sveltejs/kit';
import type { WithFetch } from '../fetch';
import type { Tag } from '$lib/types/tag';

export const getTags = async ({ fetch: _fetch }: WithFetch): Promise<Array<Tag>> => {
  type Response = {
    data: Array<Tag>;
  };

  const res = await _fetch('/api/v1/tags');

  if (!res.ok) {
    return error(500, 'oh no');
    //return await getError(res);
  }

  return await res.json().then((json: Response) => json.data);
};

export async function uploadImage(image: File) {
  const formData = new FormData();
  formData.append('file', image);
  const response = await fetch(`/api/v1/images`, {
    method: 'POST',
    body: formData,
  });

  if (!response.ok) {
    // TODO - panic!
    throw Error('oh no');
  }

  return response.json().then((json: { data: string }) => json.data);
}

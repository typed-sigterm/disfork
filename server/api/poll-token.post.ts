interface PollTokenResponse {
  access_token?: string
  error?: string
  error_description?: string
}

export default defineEventHandler(async (event) => {
  const { device_code } = await readBody<{ device_code: string }>(event);

  if (typeof device_code !== 'string')
    throw createError({ statusCode: 400, message: 'Invalid device code' });

  const res = await $fetch<PollTokenResponse>('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: new URLSearchParams({
      client_id: useRuntimeConfig().githubAppClientId,
      device_code,
      grant_type: 'urn:ietf:params:oauth:grant-type:device_code',
    }).toString(),
  });
  return res;
});

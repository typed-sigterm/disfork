interface DeviceCodeResponse {
  device_code: string
  user_code: string
  verification_uri: string
  expires_in: number
  interval: number
  error?: string
}

export default defineEventHandler(async () => {
  return await $fetch<DeviceCodeResponse>('https://github.com/login/device/code', {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    query: {
      client_id: useRuntimeConfig().githubAppClientId,
    },
  });
});

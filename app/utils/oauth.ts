export type OAuthStatus = 'idle' | 'starting' | 'awaiting' | 'polling' | 'done' | 'error';

export interface DeviceCodeResult {
  deviceCode: string
  userCode: string
  verificationUri: string
  interval: number
}

/** Request a device code. Returns display info; throws on network failure. */
export async function fetchDeviceCode(): Promise<DeviceCodeResult> {
  const res = await $fetch<{
    device_code: string
    user_code: string
    verification_uri: string
    expires_in: number
    interval: number
  }>('/api/device-code', { method: 'POST' });

  return {
    deviceCode: res.device_code,
    userCode: res.user_code,
    verificationUri: res.verification_uri,
    interval: res.interval ?? 5,
  };
}

/**
 * Poll until the user authorizes or the code expires.
 * Returns the access token; throws with a descriptive message on failure.
 */
export async function pollForToken(
  deviceCode: string,
  intervalSeconds: number,
  expiresInSeconds = 900,
): Promise<string> {
  const expiresAt = Date.now() + expiresInSeconds * 1000;
  let poll = intervalSeconds;

  while (Date.now() < expiresAt) {
    await new Promise(r => setTimeout(r, poll * 1000));

    const res = await $fetch<{ access_token?: string, error?: string }>('/api/poll-token', {
      method: 'POST',
      body: { device_code: deviceCode },
    });

    if (res.access_token)
      return res.access_token;

    if (res.error === 'slow_down')
      poll += 5;
    else if (res.error === 'expired_token')
      throw new Error('Authorization code expired');
    else if (res.error === 'access_denied')
      throw new Error('Authorization denied by user');
    // 'authorization_pending' → keep polling
  }

  throw new Error('Authorization timed out');
}

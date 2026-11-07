import { signNonce } from './ttl-crypto.mjs';
import { postJson } from './api.mjs';
import { createAppError, getErrorInfo } from './errors.mjs';
import { requestVerifiedChallenge } from './challenge.mjs';

export async function unlockDoor(memberId) {
  const challenge = await requestVerifiedChallenge(memberId);
  const serverNonceSignature = await signNonce(challenge.server_nonce);

  try {
    return await postJson('/api/door/unlock', {
      member_id: memberId,
      challenge_id: challenge.challenge_id,
      server_nonce_signature: serverNonceSignature,
    });
  } catch (error) {
    const backendError = error.backendMessage || error.message;
    const errorInfo = getErrorInfo(backendError, memberId);
    throw createAppError(errorInfo.type, errorInfo.message, backendError);
  }
}

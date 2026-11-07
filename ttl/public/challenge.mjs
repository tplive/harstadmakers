import {
  createClientNonce,
  createServerChallenge,
  getStoredServerPublicKey,
  signNonce,
  verifyServerChallenge,
} from './ttl-crypto.mjs';
import { postJson } from './api.mjs';
import { createAppError, getErrorInfo } from './errors.mjs';

export async function requestVerifiedChallenge(memberId) {
  const serverPublicKey = getStoredServerPublicKey();
  if (!serverPublicKey) {
    throw createAppError(
      'keys_missing',
      'Nøkkeldata mangler. Vennligst registrer denne nettleseren på nytt.',
      'No stored server public key',
    );
  }

  const serverChallenge = createServerChallenge();
  const clientNonce = createClientNonce();
  const clientNonceSignature = await signNonce(clientNonce);
  let data;

  try {
    data = await postJson('/api/challenge', {
      member_id: memberId,
      server_challenge: serverChallenge,
      client_nonce: clientNonce,
      client_nonce_signature: clientNonceSignature,
    });
  } catch (error) {
    const backendError = error.backendMessage || error.message;
    const errorInfo = getErrorInfo(backendError, memberId);
    throw createAppError(errorInfo.type, errorInfo.message, backendError);
  }

  if (data.client_nonce !== clientNonce) {
    throw createAppError(
      'verification_failed',
      'Server-verifisering mislyktes. Prøv på nytt.',
      'Client nonce mismatch',
    );
  }

  const verified = await verifyServerChallenge(
    serverPublicKey,
    serverChallenge,
    data.server_challenge_signature,
  );
  if (!verified) {
    throw createAppError(
      'verification_failed',
      'Server-verifisering mislyktes. Prøv på nytt.',
      'Server challenge verification failed',
    );
  }

  return data;
}

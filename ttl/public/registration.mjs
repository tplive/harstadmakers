import {
  clearStoredRegistration,
  createServerChallenge,
  exportPublicKeyJwk,
  generateMemberKeyPair,
  storeMemberId,
  storeMemberPrivateKey,
  storeServerPublicKey,
  verifyServerChallenge,
} from './ttl-crypto.mjs';
import { postJson } from './api.mjs';
import { createAppError, getErrorInfo } from './errors.mjs';
import { requestVerifiedChallenge } from './challenge.mjs';
import { translate } from './i18n.mjs';

export async function registerMember(name, email) {
  const memberKeyPair = await generateMemberKeyPair();
  const serverChallenge = createServerChallenge();
  let data;

  try {
    data = await postJson('/api/members', {
      name,
      email,
      public_key_jwk: await exportPublicKeyJwk(memberKeyPair.publicKey),
      server_challenge: serverChallenge,
    });
  } catch (error) {
    const backendError = error.backendMessage || error.message;
    const errorInfo = getErrorInfo(backendError);
    throw createAppError(errorInfo.type, errorInfo.message, backendError);
  }

  const verified = await verifyServerChallenge(
    data.server_public_key,
    serverChallenge,
    data.server_challenge_signature,
  );
  if (!verified) {
    throw createAppError(
      'verification_failed',
      translate('error.registrationVerification'),
      'Server challenge verification failed during registration',
    );
  }

  await storeMemberPrivateKey(memberKeyPair.privateKey);
  storeServerPublicKey(data.server_public_key);
  storeMemberId(data.member.id);
  return data.member;
}

export async function checkRegistration(memberId) {
  return requestVerifiedChallenge(memberId);
}

export function deleteRegistration() {
  clearStoredRegistration();
}

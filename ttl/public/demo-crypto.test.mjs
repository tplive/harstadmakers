import test from 'node:test';
import assert from 'node:assert/strict';
import { constants, generateKeyPairSync, publicEncrypt } from 'node:crypto';
import {
  MEMBER_PRIVATE_KEY_STORAGE_KEY,
  decryptChallengeNonce,
  verifyServerChallenge,
} from './demo-crypto.mjs';

const memoryStore = new Map();
globalThis.localStorage = {
  getItem(key) {
    return memoryStore.has(key) ? memoryStore.get(key) : null;
  },
  setItem(key, value) {
    memoryStore.set(key, String(value));
  },
  removeItem(key) {
    memoryStore.delete(key);
  },
};

const { privateKey, publicKey } = generateKeyPairSync('rsa', {
  modulusLength: 2048,
  publicKeyEncoding: { type: 'spki', format: 'pem' },
  privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
});
const privateKeyPkcs8 = privateKey
  .replace('-----BEGIN PRIVATE KEY-----\n', '')
  .replace('\n-----END PRIVATE KEY-----\n', '')
  .replace(/\n/g, '');

memoryStore.set(MEMBER_PRIVATE_KEY_STORAGE_KEY, privateKeyPkcs8);

test('demo private key can decrypt a challenge encrypted by the matching public key', async () => {
  const plaintext = 'challenge-nonce-demo-123';
  const encrypted = publicEncrypt(
    {
      key: publicKey,
      padding: constants.RSA_PKCS1_OAEP_PADDING,
      oaepHash: 'sha256',
    },
    Buffer.from(plaintext, 'utf8')
  );

  const decrypted = await decryptChallengeNonce(encrypted.toString('base64'));
  assert.equal(decrypted, plaintext);
});

test('browser can verify an ed25519 server challenge from an OpenSSH public key', async () => {
  const algorithm = 'ssh-ed25519';
  const keyPair = await crypto.subtle.generateKey('Ed25519', true, ['sign', 'verify']);
  const rawPublicKey = new Uint8Array(await crypto.subtle.exportKey('raw', keyPair.publicKey));
  const algorithmBytes = new TextEncoder().encode(algorithm);

  const wire = new Uint8Array(4 + algorithmBytes.length + 4 + rawPublicKey.length);
  const view = new DataView(wire.buffer);
  let offset = 0;

  view.setUint32(offset, algorithmBytes.length);
  offset += 4;
  wire.set(algorithmBytes, offset);
  offset += algorithmBytes.length;
  view.setUint32(offset, rawPublicKey.length);
  offset += 4;
  wire.set(rawPublicKey, offset);

  const opensshPublicKey = `ssh-ed25519 ${Buffer.from(wire).toString('base64')} test@example`;
  const challenge = 'verify-the-server';
  const signature = await crypto.subtle.sign(
    'Ed25519',
    keyPair.privateKey,
    new TextEncoder().encode(challenge)
  );

  const verified = await verifyServerChallenge(
    opensshPublicKey,
    challenge,
    Buffer.from(signature).toString('base64')
  );

  assert.equal(verified, true);
});

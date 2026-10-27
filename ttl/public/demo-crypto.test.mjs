import test from 'node:test';
import assert from 'node:assert/strict';
import {
  generateMemberKeyPair,
  getStoredPrivateKey,
  storeMemberPrivateKey,
  signNonce,
} from './demo-crypto.mjs';

const memoryStore = new Map();
globalThis.localStorage = {
  getItem: (key) => memoryStore.get(key) ?? null,
  setItem: (key, value) => memoryStore.set(key, String(value)),
  removeItem: (key) => memoryStore.delete(key),
};

test('browser signs a client nonce with its Ed25519 private key', async () => {
  const keyPair = await generateMemberKeyPair();
  await storeMemberPrivateKey(keyPair.privateKey);
  const nonce = 'client-nonce-demo-123';
  const signature = await signNonce(nonce);
  const verified = await crypto.subtle.verify(
    'Ed25519',
    keyPair.publicKey,
    Uint8Array.from(atob(signature), (char) => char.charCodeAt(0)),
    new TextEncoder().encode(nonce)
  );
  assert.equal(verified, true);
  assert.ok(await getStoredPrivateKey());
});

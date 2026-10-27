export const MEMBER_PRIVATE_KEY_STORAGE_KEY = 'harstad-makers-private-key';
export const MEMBER_ID_STORAGE_KEY = 'harstad-makers-member-id';
export const SERVER_PUBLIC_KEY_STORAGE_KEY = 'harstad-makers-server-public-key';
const fallbackStorage = {
  values: new Map(),
  getItem(key) {
    return this.values.has(key) ? this.values.get(key) : null;
  },
  setItem(key, value) {
    this.values.set(key, String(value));
  },
  removeItem(key) {
    this.values.delete(key);
  },
};

function getStorage() {
  return globalThis.localStorage ?? fallbackStorage;
}

function base64UrlToUint8Array(value) {
  const normalized = value.replace(/-/g, '+').replace(/_/g, '/');
  const padded = normalized + '='.repeat((4 - (normalized.length % 4)) % 4);
  const binary = atob(padded);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function base64ToUint8Array(value) {
  const normalized = value.replace(/\s+/g, '');
  const binary = atob(normalized);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function uint8ArrayToBase64(bytes) {
  let binary = '';
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte);
  });
  return btoa(binary);
}

export async function generateMemberKeyPair() {
  return crypto.subtle.generateKey(
    {
      name: 'Ed25519',
    },
    true,
    ['sign', 'verify']
  );
}

export function createServerChallenge() {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  return uint8ArrayToBase64(bytes);
}

export function createClientNonce() {
  return createServerChallenge();
}

export async function signNonce(nonce) {
  const privateKey = await getStoredPrivateKey();
  if (!privateKey) {
    throw new Error('No member private key found in browser storage. Register a member first.');
  }
  const signature = await crypto.subtle.sign('Ed25519', privateKey, new TextEncoder().encode(nonce));
  return uint8ArrayToBase64(new Uint8Array(signature));
}

export async function exportPrivateKeyJwk(privateKey) {
  return crypto.subtle.exportKey('jwk', privateKey);
}

export async function exportPublicKeyJwk(publicKey) {
  return crypto.subtle.exportKey('jwk', publicKey);
}

export async function getStoredPrivateKey() {
  const storage = getStorage();
  const stored = storage.getItem(MEMBER_PRIVATE_KEY_STORAGE_KEY);
  if (!stored) {
    return null;
  }

  try {
    const keyBytes = base64UrlToUint8Array(stored);
    return crypto.subtle.importKey('pkcs8', keyBytes, { name: 'Ed25519' }, false, ['sign']);
  } catch (error) {
    console.warn('Stored private key is invalid, removing it.', error);
    storage.removeItem(MEMBER_PRIVATE_KEY_STORAGE_KEY);
    return null;
  }
}

export async function storeMemberPrivateKey(privateKey) {
  const storage = getStorage();
  const pkcs8Bytes = await crypto.subtle.exportKey('pkcs8', privateKey);
  const encoded = uint8ArrayToBase64(new Uint8Array(pkcs8Bytes));
  storage.setItem(MEMBER_PRIVATE_KEY_STORAGE_KEY, encoded);
}

export function storeServerPublicKey(publicKey) {
  if (publicKey) {
    getStorage().setItem(SERVER_PUBLIC_KEY_STORAGE_KEY, publicKey);
  }
}

export function getStoredServerPublicKey() {
  return getStorage().getItem(SERVER_PUBLIC_KEY_STORAGE_KEY) || '';
}

function extractEd25519RawPublicKey(publicKey) {
  const [algorithm, payload] = String(publicKey || '').trim().split(/\s+/, 3);

  if (algorithm !== 'ssh-ed25519' || !payload) {
    throw new Error('Server public key is missing or not an ssh-ed25519 key.');
  }

  const wireBytes = base64ToUint8Array(payload);
  const view = new DataView(wireBytes.buffer, wireBytes.byteOffset, wireBytes.byteLength);
  let offset = 0;

  const readChunk = () => {
    if (offset + 4 > wireBytes.byteLength) {
      throw new Error('Server public key payload is truncated.');
    }

    const length = view.getUint32(offset);
    offset += 4;

    if (offset + length > wireBytes.byteLength) {
      throw new Error('Server public key payload is invalid.');
    }

    const chunk = wireBytes.slice(offset, offset + length);
    offset += length;
    return chunk;
  };

  const algorithmBytes = readChunk();
  const algorithmName = new TextDecoder().decode(algorithmBytes);
  if (algorithmName !== 'ssh-ed25519') {
    throw new Error('Server public key payload algorithm mismatch.');
  }

  const keyBytes = readChunk();
  if (keyBytes.byteLength !== 32) {
    throw new Error('Server public key has an unexpected length.');
  }

  return keyBytes;
}

export async function verifyServerChallenge(publicKey, challenge, signatureBase64) {
  const rawKey = extractEd25519RawPublicKey(publicKey);
  const verifyKey = await crypto.subtle.importKey('raw', rawKey, 'Ed25519', false, ['verify']);
  const signature = base64ToUint8Array(signatureBase64);

  return crypto.subtle.verify(
    'Ed25519',
    verifyKey,
    signature,
    new TextEncoder().encode(challenge)
  );
}

export function storeMemberId(memberId) {
  if (memberId) {
    getStorage().setItem(MEMBER_ID_STORAGE_KEY, memberId);
  }
}

export function getStoredMemberId() {
  return getStorage().getItem(MEMBER_ID_STORAGE_KEY) || '';
}

export function clearStoredRegistration() {
  const storage = getStorage();
  storage.removeItem(MEMBER_PRIVATE_KEY_STORAGE_KEY);
  storage.removeItem(MEMBER_ID_STORAGE_KEY);
  storage.removeItem(SERVER_PUBLIC_KEY_STORAGE_KEY);
}

export function setMemberIdField(value) {
  const memberIdField = document.getElementById('member-id');
  if (memberIdField) {
    memberIdField.value = value || '';
  }
}

export function setChallengeDecryptedNonce(value) {
  const decryptedNonceField = document.getElementById('decrypted-nonce');
  if (decryptedNonceField) {
    decryptedNonceField.value = value || '';
  }
}

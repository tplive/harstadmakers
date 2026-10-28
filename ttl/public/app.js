const DEFAULT_API_BASE = 'http://localhost:3000';

function resolveApiBase() {
  const manualBase = new URLSearchParams(window.location.search).get('api_base')
    || window.localStorage.getItem('api_base');

  if (manualBase) {
    return manualBase.replace(/\/+$/, '');
  }

  const hostname = window.location.hostname;

  if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '0.0.0.0') {
    return DEFAULT_API_BASE;
  }

  const protocol = window.location.protocol === 'https:' ? 'https' : 'http';
  return `${protocol}://${hostname}:3000`;
}

import {
  clearStoredRegistration,
  createServerChallenge,
  createClientNonce,
  signNonce,
  exportPublicKeyJwk,
  generateMemberKeyPair,
  getStoredServerPublicKey,
  getStoredMemberId,
  storeMemberId,
  storeMemberPrivateKey,
  storeServerPublicKey,
  verifyServerChallenge,
} from './ttl-crypto.mjs';

const API_BASE = resolveApiBase();

// DOM elements
const statusMessageEl = document.getElementById('status-message');
const registrationPanelEl = document.getElementById('registration-panel');
const memberPanelEl = document.getElementById('member-panel');
const memberPanelCopyEl = document.getElementById('member-panel-copy');
const registeredMemberIdEl = document.getElementById('registered-member-id');
const registerMemberNameField = document.getElementById('register-member-name');
const registerMemberEmailField = document.getElementById('register-member-email');
const registerBrowserButton = document.getElementById('register-browser');
const unlockDoorButton = document.getElementById('unlock-door');
const deleteRegistrationButton = document.getElementById('delete-registration');

// State
let currentMemberId = null;
let currentChallengeId = null;
let isProcessing = false;

// Helper functions
function setHidden(element, hidden) {
  if (element) {
    element.classList.toggle('hidden', hidden);
  }
}

function setStatus(message, type = 'neutral') {
  console.log(`Status: ${message} (type: ${type})`);
  statusMessageEl.textContent = message;
  statusMessageEl.className = `status-box ${type}`;
}

function showLoading() {
  setStatus('Venter...', 'neutral');
  isProcessing = true;
  unlockDoorButton.disabled = true;
  registerBrowserButton.disabled = true;
}

function hideLoading() {
  isProcessing = false;
  unlockDoorButton.disabled = false;
  registerBrowserButton.disabled = false;
}

// Check if we have a stored registration and try to get a challenge
async function checkRegistrationStatus() {
  const storedMemberId = getStoredMemberId();
  console.log('Checking registration status...', { storedMemberId });
  
  if (!storedMemberId) {
    // No registration in localStorage
    console.log('No registration found in localStorage');
    currentMemberId = null;
    setStatus('Du må registrere denne nettleseren først.', 'neutral');
    setHidden(registrationPanelEl, false);
    setHidden(memberPanelEl, true);
    registeredMemberIdEl.textContent = '';
    return;
  }

  // We have a registration in localStorage, try to get a challenge
  console.log('Found registration in localStorage, attempting challenge...');
  currentMemberId = storedMemberId;
  registeredMemberIdEl.textContent = storedMemberId;
  setStatus('Sjekker registrering på server...', 'neutral');
  
  try {
    await attemptChallengeFlow(storedMemberId);
  } catch (error) {
    console.error('Challenge flow failed:', error);
    
    // Extract error info
    const errorMsg = error.message || error.toString() || 'Klarte ikke å verifisere registreringen.';
    const errorType = error.type || 'unknown';
    const originalError = error.original || errorMsg;
    
    console.error('Error type:', errorType, 'Message:', errorMsg);
    setStatus(errorMsg, 'error');
    
    // Determine which panel to show based on error type
    if (errorType === 'registration_missing' || errorType === 'keys_missing') {
      // Registration data is missing or invalid - need to re-register
      setHidden(registrationPanelEl, false);
      setHidden(memberPanelEl, true);
      memberPanelCopyEl.textContent = 'Nettleseren er registrert og klar til bruk.';
    } else {
      // Member is registered but has an issue (inactive, network, challenge, etc.)
      setHidden(registrationPanelEl, true);
      setHidden(memberPanelEl, false);
      
      // Update panel copy based on error type
      if (errorType === 'member_expired') {
        memberPanelCopyEl.textContent = 'Tilgangen er utløpt og deaktivert. Kontakt administrator for reaktivering.';
      } else if (errorType === 'member_inactive') {
        memberPanelCopyEl.textContent = 'Registreringen venter på godkjenning fra administrator.';
      } else {
        memberPanelCopyEl.textContent = 'Nettleseren er registrert og klar til bruk.';
      }
    }
  }
}

// Attempt the full challenge flow to verify registration is valid
async function attemptChallengeFlow(memberId) {
  console.log('Starting challenge flow for member:', memberId);
  showLoading();
  
  try {
    const serverPublicKey = getStoredServerPublicKey();
    if (!serverPublicKey) {
      throw { type: 'keys_missing', message: 'Nøkkeldata mangler. Vennligst registrer denne nettleseren på nytt.', original: 'No stored server public key' };
    }

    const serverChallenge = createServerChallenge();
    const clientNonce = createClientNonce();
    const clientNonceSignature = await signNonce(clientNonce);
    console.log('Created server challenge:', serverChallenge.substring(0, 20) + '...');
    
    const response = await fetch(`${API_BASE}/api/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ 
        member_id: memberId, 
        server_challenge: serverChallenge,
        client_nonce: clientNonce,
        client_nonce_signature: clientNonceSignature
      }),
    });

    const data = await response.json();
    console.log('Challenge response received:', data);

    if (!response.ok) {
      const backendError = data.error || 'Ukjent feil';
      console.error('Challenge request failed:', backendError);
      const errorInfo = getErrorInfo(backendError, memberId);
      console.error('Error type:', errorInfo.type);
      throw { type: errorInfo.type, message: errorInfo.message, original: backendError };
    }

    if (data.client_nonce !== clientNonce) {
      throw { type: 'verification_failed', message: 'Server-verifisering mislyktes. Prøv på nytt.', original: 'Client nonce mismatch' };
    }

    // Verify server signature
    const verified = await verifyServerChallenge(
      serverPublicKey,
      serverChallenge,
      data.server_challenge_signature
    );

    if (!verified) {
      throw { type: 'verification_failed', message: 'Server-verifisering mislyktes. Prøv på nytt.', original: 'Server challenge verification failed' };
    }

    // Store challenge ID for later use
    currentChallengeId = data.challenge_id;
    console.log('Challenge verified, storing challenge_id:', currentChallengeId);
    
    // Registration is valid and member is active
    console.log('Registration verified and member is active');
    setStatus('Registrering bekreftet. Klar til å låse opp.', 'success');
    setHidden(registrationPanelEl, true);
    setHidden(memberPanelEl, false);
    
  } catch (error) {
    console.error('Challenge flow error:', error);
    
    // Check if error has a type (from our error handling) or if it's a regular Error
    if (error.type) {
      // This is one of our structured errors
      throw error;
    }
    
    // Otherwise, it's a regular Error object - convert it
    const errorMsg = error.message || String(error);
    const errorInfo = getErrorInfo(errorMsg, memberId);
    throw { type: errorInfo.type, message: errorInfo.message, original: errorMsg };
  } finally {
    hideLoading();
  }
}

// Map backend error messages to error types and user-friendly messages
function getErrorInfo(backendError, memberId) {
  const errorLower = (backendError || '').toLowerCase();
  
  // Check for specific backend error messages
  if (errorLower.includes('member not found')) {
    return {
      type: 'registration_missing',
      message: `Registreringen din (${memberId}) finnes ikke på serveren. Den kan ha blitt slettet. Vennligst registrer på nytt.`
    };
  }
  if (errorLower.includes('expired') || errorLower.includes('utløpt')) {
    return {
      type: 'member_expired',
      message: `Tilgangen din (${memberId}) er utløpt og deaktivert. Kontakt administrator for reaktivering.`
    };
  }
  if (errorLower.includes('member is inactive') || errorLower.includes('inactive')) {
    return {
      type: 'member_inactive',
      message: `Din registrering (${memberId}) er ikke aktivert enda. Vennligst vent på at administrator godkjenner den.`
    };
  }
  if (errorLower.includes('no stored private key') || errorLower.includes('private key') || errorLower.includes('not found in browser storage')) {
    return {
      type: 'keys_missing',
      message: 'Nøkkeldata mangler. Vennligst registrer denne nettleseren på nytt.'
    };
  }
  if (errorLower.includes('server identitetssjekk') || errorLower.includes('signature') || errorLower.includes('verifisering')) {
    return {
      type: 'verification_failed',
      message: 'Server-verifisering mislyktes. Prøv på nytt.'
    };
  }
  if (errorLower.includes('network') || errorLower.includes('fetch') || errorLower.includes('failed to fetch') || errorLower.includes('koble til')) {
    return {
      type: 'network_error',
      message: 'Kunne ikke koble til serveren. Sjekk at du er koblet til riktig Wi-Fi-nettverk.'
    };
  }
  if (errorLower.includes('challenge') || errorLower.includes('nonce') || errorLower.includes('expired')) {
    return {
      type: 'challenge_error',
      message: backendError || 'Utfordringsfeil. Prøv på nytt.'
    };
  }
  
  return {
    type: 'unknown',
    message: backendError || 'Ukjent feil.'
  };
}

// Register a new member/browser
async function registerMember() {
  const name = registerMemberNameField.value.trim();
  const email = registerMemberEmailField.value.trim();

  if (!name || !email) {
    setStatus('Fullt navn og e-post må fylles ut.', 'error');
    return;
  }

  showLoading();
  
  try {
    console.log('Starting registration for:', name, email);
    
    const memberKeyPair = await generateMemberKeyPair();
    const publicKeyJwk = await exportPublicKeyJwk(memberKeyPair.publicKey);
    const serverChallenge = createServerChallenge();

    console.log('Generated key pair and server challenge');

    const response = await fetch(`${API_BASE}/api/members`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name,
        email,
        public_key_jwk: publicKeyJwk,
        server_challenge: serverChallenge,
      }),
    });

    const data = await response.json();
    console.log('Registration response:', data);

    if (!response.ok) {
      const backendError = data.error || 'Registrering mislyktes.';
      const errorInfo = getErrorInfo(backendError, '');
      throw { type: errorInfo.type, message: errorInfo.message, original: backendError };
    }

    // Verify server signature
    const verified = await verifyServerChallenge(
      data.server_public_key,
      serverChallenge,
      data.server_challenge_signature
    );

    if (!verified) {
      throw { type: 'verification_failed', message: 'Server-verifisering mislyktes under registrering.', original: 'Server challenge verification failed during registration' };
    }

    // Store registration data
    await storeMemberPrivateKey(memberKeyPair.privateKey);
    storeServerPublicKey(data.server_public_key);
    storeMemberId(data.member.id);
    
    currentMemberId = data.member.id;
    registeredMemberIdEl.textContent = data.member.id;
    
    console.log('Registration complete, member ID:', currentMemberId);
    setStatus(`Registrering fullført! Vent på godkjenning fra administrator.`, 'success');
    memberPanelCopyEl.textContent = 'Registreringen venter på godkjenning fra administrator.';
    
    // Clear form
    registerMemberNameField.value = '';
    registerMemberEmailField.value = '';
    
    // Show member panel but with info that it's pending
    setHidden(registrationPanelEl, true);
    setHidden(memberPanelEl, false);
    
  } catch (error) {
    console.error('Registration error:', error);
    setStatus(error.message || 'Registrering mislyktes.', 'error');
  } finally {
    hideLoading();
  }
}

// Unlock the door - full flow
async function unlockDoor() {
  if (!currentMemberId) {
    setStatus('Du må registrere nettleseren først.', 'error');
    return;
  }

  showLoading();
  setStatus('Ber om utfordring...', 'neutral');
  
  try {
    console.log('Starting unlock flow for member:', currentMemberId);
    
    const serverPublicKey = getStoredServerPublicKey();
    if (!serverPublicKey) {
      throw { type: 'keys_missing', message: 'Nøkkeldata mangler. Vennligst registrer denne nettleseren på nytt.', original: 'No stored server public key' };
    }

    const serverChallenge = createServerChallenge();
    const clientNonce = createClientNonce();
    const clientNonceSignature = await signNonce(clientNonce);
    
    // Request challenge
    const challengeResponse = await fetch(`${API_BASE}/api/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ 
        member_id: currentMemberId, 
        server_challenge: serverChallenge,
        client_nonce: clientNonce,
        client_nonce_signature: clientNonceSignature
      }),
    });

    const challengeData = await challengeResponse.json();
    console.log('Challenge received:', challengeData);

    if (!challengeResponse.ok) {
      const backendError = challengeData.error || 'Challenge mislyktes';
      const errorInfo = getErrorInfo(backendError, currentMemberId);
      throw { type: errorInfo.type, message: errorInfo.message, original: backendError };
    }

    if (challengeData.client_nonce !== clientNonce) {
      throw { type: 'verification_failed', message: 'Server-verifisering mislyktes.', original: 'Client nonce mismatch' };
    }

    // Verify server signature
    const verified = await verifyServerChallenge(
      serverPublicKey,
      serverChallenge,
      challengeData.server_challenge_signature
    );

    if (!verified) {
      throw { type: 'verification_failed', message: 'Server-verifisering mislyktes.', original: 'Server challenge verification failed' };
    }

    currentChallengeId = challengeData.challenge_id;
    console.log('Challenge verified, signing server nonce...');
    
    const serverNonceSignature = await signNonce(challengeData.server_nonce);

    // Unlock door
    setStatus('Låser opp døra...', 'neutral');
    console.log('Sending unlock request...');
    
    const unlockResponse = await fetch(`${API_BASE}/api/door/unlock`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        member_id: currentMemberId,
        challenge_id: currentChallengeId,
        server_nonce_signature: serverNonceSignature,
      }),
    });

    const unlockData = await unlockResponse.json();
    console.log('Unlock response:', unlockData);

    if (!unlockResponse.ok) {
      const backendError = unlockData.error || 'Klarte ikke å låse opp døra.';
      const errorInfo = getErrorInfo(backendError, currentMemberId);
      throw { type: errorInfo.type, message: errorInfo.message, original: backendError };
    }

    console.log('Door unlocked successfully!');
    setStatus('Døra er låst opp!', 'success');
    
    // Clear challenge ID after use
    currentChallengeId = null;
    
    // Auto-hide success message after 3 seconds
    setTimeout(() => {
      if (!isProcessing) {
        setStatus('Klar til å låse opp på nytt.', 'neutral');
      }
    }, 3000);
    
  } catch (error) {
    console.error('Unlock error:', error);
    setStatus(error.message || 'Klarte ikke å låse opp døra.', 'error');
  } finally {
    hideLoading();
  }
}

// Delete local registration
function deleteRegistration() {
  console.log('Deleting local registration');
  clearStoredRegistration();
  currentMemberId = null;
  currentChallengeId = null;
  registeredMemberIdEl.textContent = '';
  memberPanelCopyEl.textContent = 'Nettleseren er registrert og klar til bruk.';
  
  setHidden(registrationPanelEl, false);
  setHidden(memberPanelEl, true);
  setStatus('Registrering slettet. Du kan registrere på nytt når som helst.', 'neutral');
}

// Event listeners
registerBrowserButton.addEventListener('click', registerMember);
unlockDoorButton.addEventListener('click', unlockDoor);
deleteRegistrationButton.addEventListener('click', deleteRegistration);

// Initialize on page load
console.log('Initializing app...');
setStatus('Sjekker registreringsstatus...', 'neutral');
setHidden(registrationPanelEl, true);
setHidden(memberPanelEl, true);
checkRegistrationStatus();

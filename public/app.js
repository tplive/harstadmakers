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
  decryptChallengeNonce,
  exportPublicKeyJwk,
  generateMemberKeyPair,
  getStoredServerPublicKey,
  getStoredMemberId,
  setChallengeDecryptedNonce,
  setMemberIdField,
  storeMemberId,
  storeMemberPrivateKey,
  storeServerPublicKey,
  verifyServerChallenge,
} from './demo-crypto.mjs';

const API_BASE = resolveApiBase();

const memberListEl = document.getElementById('member-list');
const debugSummaryEl = document.getElementById('debug-summary');
const statusBox = document.getElementById('status-box');
const memberStatusEl = document.getElementById('member-status');
const challengeOutput = document.getElementById('challenge-output');
const debugActionsEl = document.getElementById('debug-actions');
const registerPanelEl = document.getElementById('register-panel');
const memberPanelEl = document.getElementById('member-panel');
const debugPanelEl = document.getElementById('debug-panel');
const registeredMemberIdEl = document.getElementById('registered-member-id');
const memberPanelCopyEl = document.getElementById('member-panel-copy');
const memberIdField = document.getElementById('member-id');
const registerMemberNameField = document.getElementById('register-member-name');
const registerMemberEmailField = document.getElementById('register-member-email');
const debugMemberNameField = document.getElementById('debug-member-name');
const debugMemberEmailField = document.getElementById('debug-member-email');
const challengeIdField = document.getElementById('challenge-id');
const decryptedNonceField = document.getElementById('decrypted-nonce');
const registerBrowserButton = document.getElementById('register-browser');
const quickUnlockButton = document.getElementById('unlock-door-quick');
const deleteRegistrationButton = document.getElementById('delete-browser-registration');

const viewState = {
  members: [],
  currentMember: null,
  isDebug: false,
  isRegistered: false,
};

const existingMemberId = getStoredMemberId();
if (existingMemberId) {
  setMemberIdField(existingMemberId);
}

function setHidden(element, hidden) {
  if (element) {
    element.classList.toggle('hidden', hidden);
  }
}

function updateCurrentMember(members = viewState.members) {
  viewState.members = members;
  const storedMemberId = getStoredMemberId();
  viewState.isRegistered = Boolean(storedMemberId);
  viewState.currentMember = Array.isArray(members)
    ? members.find((member) => member.id === storedMemberId) || null
    : null;
  viewState.isDebug = Boolean(viewState.currentMember?.is_debug);
}

function renderMode() {
  const storedMemberId = getStoredMemberId();
  registeredMemberIdEl.textContent = storedMemberId || translations.siteTitle;

  if (viewState.isDebug) {
    setHidden(debugSummaryEl, false);
    setHidden(memberStatusEl, true);
    setHidden(debugActionsEl, false);
    setHidden(debugPanelEl, false);
    setHidden(registerPanelEl, true);
    setHidden(memberPanelEl, true);
    return;
  }

  setHidden(debugSummaryEl, true);
  setHidden(memberStatusEl, false);
  setHidden(debugActionsEl, true);
  setHidden(debugPanelEl, true);

  if (!viewState.isRegistered) {
    setHidden(registerPanelEl, false);
    setHidden(memberPanelEl, true);
    memberPanelCopyEl.textContent = translations.memberPanelDescription;
    return;
  }

  setHidden(registerPanelEl, true);
  setHidden(memberPanelEl, false);

  if (viewState.currentMember && !viewState.currentMember.active) {
    memberPanelCopyEl.textContent = 'Nettleseren er registrert, men venter fortsatt på godkjenning før døra kan låses opp.';
  } else {
    memberPanelCopyEl.textContent = translations.memberPanelDescription;
  }
}

function setStatus(message, type = 'neutral') {
  statusBox.textContent = message;
  statusBox.className = `status-box ${type}`;
  memberStatusEl.textContent = message;
  memberStatusEl.className = `status-box ${type}`;

  if (!viewState.isDebug) {
    memberStatusEl.classList.remove('hidden');
  }
}

function renderMembers(members) {
  memberListEl.innerHTML = '';

  if (!Array.isArray(members) || members.length === 0) {
    memberListEl.innerHTML = '<li>Ingen medlemmer enda.</li>';
    return;
  }

  members.forEach((member) => {
    const item = document.createElement('li');
    const role = member.is_debug ? 'debug' : (member.active ? 'aktiv' : 'venter');
    item.textContent = `${member.name} · ${member.email} · ${role}`;
    memberListEl.appendChild(item);
  });
}

async function loadMembers({ silent = false } = {}) {
  try {
    const response = await fetch(`${API_BASE}/api/members`);
    const members = await response.json();

    updateCurrentMember(members);
    renderMembers(members);
    renderMode();

    if (!silent && viewState.isDebug) {
      setStatus(`Lastet ${members.length} medlem(mer).`, 'success');
    }
  } catch (error) {
    updateCurrentMember([]);
    renderMode();

    if (!silent) {
      setStatus(`Kunne ikke koble til backend: ${error.message}`, 'error');
    }
  }
}

async function registerMember() {
  const activeNameField = viewState.isDebug ? debugMemberNameField : registerMemberNameField;
  const activeEmailField = viewState.isDebug ? debugMemberEmailField : registerMemberEmailField;
  const name = activeNameField.value.trim();
  const email = activeEmailField.value.trim();

  if (!name || !email) {
    setStatus(translations.labelFullName + ' og ' + translations.labelEmail + ' må fylles ut for å registrere medlemmet.', 'error');
    return;
  }

  try {
    const memberKeyPair = await generateMemberKeyPair();
    const publicKeyJwk = await exportPublicKeyJwk(memberKeyPair.publicKey);
    const serverChallenge = createServerChallenge();

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
    if (!response.ok) {
      throw new Error(data.error || 'Registrering mislyktes.');
    }

    const verified = await verifyServerChallenge(
      data.server_public_key,
      serverChallenge,
      data.server_challenge_signature
    );

    if (!verified) {
      throw new Error('Server identitetssjekk mislyktes under registrering.');
    }

    await storeMemberPrivateKey(memberKeyPair.privateKey);
    storeServerPublicKey(data.server_public_key);
    storeMemberId(data.member.id);
    setMemberIdField(data.member.id);
    updateCurrentMember(viewState.members);
    renderMode();
    setStatus(`Registrert medlem ${data.member.id}. Nettleser og server identiteter er nå koblet sammen.`, 'success');
    activeNameField.value = data.member.name || name;
    activeEmailField.value = data.member.email || email;
    challengeOutput.textContent = JSON.stringify(data, null, 2);
    await loadMembers({ silent: true });
  } catch (error) {
    setStatus(error.message, 'error');
  }
}

async function requestChallenge() {
  const memberId = memberIdField.value.trim();

  if (!memberId) {
    setStatus(translations.labelMemberId + ' må fylles ut.', 'error');
    return;
  }

  try {
    const serverPublicKey = getStoredServerPublicKey();
    if (!serverPublicKey) {
      throw new Error(translations.siteTitle + ': ' + 'Ingen lagret server nøkkel funnet. Registrer denne nettleseren på nytt først.');
    }

    const serverChallenge = createServerChallenge();
    const response = await fetch(`${API_BASE}/api/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ member_id: memberId, server_challenge: serverChallenge }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || 'Challenge forespørsel mislyktes.');
    }

    const verified = await verifyServerChallenge(
      serverPublicKey,
      serverChallenge,
      data.server_challenge_signature
    );

    if (!verified) {
      throw new Error('Server identitetssjekk mislyktes for dette opplåsingsforsøket.');
    }

    challengeIdField.value = data.challenge_id;
    challengeOutput.textContent = JSON.stringify(data, null, 2);

    const decryptedNonce = await decryptChallengeNonce(data.encrypted_nonce);
    setChallengeDecryptedNonce(decryptedNonce);
    setStatus('Challenge utstedt og dekryptert med den lagrede nettleser nøkkelen.', 'success');
  } catch (error) {
    setStatus(error.message, 'error');
  }
}

async function unlockDoor() {
  const memberId = memberIdField.value.trim();
  const challengeId = challengeIdField.value.trim();
  const decryptedNonce = decryptedNonceField.value.trim();

  if (!memberId || !challengeId || !decryptedNonce) {
    setStatus(translations.labelMemberId + ', Challenge ID og ' + translations.labelDecryptedNonce + ' må fylles ut.', 'error');
    return;
  }

  try {
    const response = await fetch(`${API_BASE}/api/door/unlock`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        member_id: memberId,
        challenge_id: challengeId,
        decrypted_nonce: decryptedNonce,
      }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || 'Dør opplåsingsforespørsel mislyktes.');
    }

    setStatus(`${data.message}`, 'success');
    challengeOutput.textContent = JSON.stringify(data, null, 2);
  } catch (error) {
    setStatus(error.message, 'error');
  }
}

async function unlockDoorFromStoredRegistration() {
  const storedMemberId = getStoredMemberId();

  if (!storedMemberId) {
    setStatus('Nettleseren er ikke registrert enda.', 'error');
    renderMode();
    return;
  }

  setMemberIdField(storedMemberId);

  try {
    quickUnlockButton.disabled = true;
    await requestChallenge();

    const challengeId = challengeIdField.value.trim();
    const decryptedNonce = decryptedNonceField.value.trim();
    if (!challengeId || !decryptedNonce) {
      throw new Error('Challenge flyt ble ikke fullført.');
    }

    await unlockDoor();
  } catch (error) {
    setStatus(error.message, 'error');
  } finally {
    quickUnlockButton.disabled = false;
  }
}

function deleteBrowserRegistration() {
  clearStoredRegistration();
  setMemberIdField('');
  setChallengeDecryptedNonce('');
  challengeIdField.value = '';
  challengeOutput.textContent = translations.challengeNotReceived;
  updateCurrentMember(viewState.members);
  renderMode();
  setStatus('Nettleserregistreringen ble slettet lokalt. Ingen data ble fjernet fra backend.', 'success');
}

document.getElementById('load-members').addEventListener('click', loadMembers);
document.getElementById('register-member').addEventListener('click', registerMember);
document.getElementById('request-challenge').addEventListener('click', requestChallenge);
document.getElementById('unlock-door').addEventListener('click', unlockDoor);
registerBrowserButton.addEventListener('click', registerMember);
quickUnlockButton.addEventListener('click', unlockDoorFromStoredRegistration);
deleteRegistrationButton.addEventListener('click', deleteBrowserRegistration);

renderMode();
loadMembers({ silent: true });

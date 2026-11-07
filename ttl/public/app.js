import { getStoredMemberId } from './ttl-crypto.mjs';
import { checkRegistration, deleteRegistration, registerMember } from './registration.mjs';
import { unlockDoor } from './unlock.mjs';
import { getErrorInfo } from './errors.mjs';
import {
  clearRegistrationForm,
  initialize,
  isProcessing,
  onDelete,
  onRegister,
  onUnlock,
  registrationInput,
  setMemberId,
  setProcessing,
  setStatus,
  showMemberPanel,
  showRegistrationPanel,
} from './ui.mjs';

let currentMemberId = null;

function showRegistrationState() {
  currentMemberId = null;
  setMemberId('');
  showRegistrationPanel();
  setStatus('Du må registrere denne nettleseren først.', 'neutral');
}

async function checkRegistrationStatus() {
  const memberId = getStoredMemberId();
  if (!memberId) {
    showRegistrationState();
    return;
  }

  currentMemberId = memberId;
  setMemberId(memberId);
  setStatus('Sjekker registrering på server...', 'neutral');

  try {
    await checkRegistration(memberId);
    setStatus('Registrering bekreftet. Klar til å låse opp.', 'success');
    showMemberPanel();
  } catch (error) {
    const errorInfo = error.type
      ? error
      : getErrorInfo(error.message, memberId);
    setStatus(errorInfo.message, 'error');
    if (error.type === 'registration_missing' || error.type === 'keys_missing') {
      showRegistrationPanel();
    } else if (error.type === 'member_expired') {
      showMemberPanel('Tilgangen er utløpt og deaktivert. Kontakt administrator for reaktivering.');
    } else if (error.type === 'member_inactive') {
      showMemberPanel('Registreringen venter på godkjenning fra administrator.');
    } else {
      showMemberPanel();
    }
  }
}

async function handleRegistration() {
  const { name, email } = registrationInput();
  if (!name || !email) {
    setStatus('Fullt navn og e-post må fylles ut.', 'error');
    return;
  }

  setProcessing(true);
  try {
    const member = await registerMember(name, email);
    currentMemberId = member.id;
    setMemberId(member.id);
    clearRegistrationForm();
    showMemberPanel('Registreringen venter på godkjenning fra administrator.');
    setStatus('Registrering fullført! Vent på godkjenning fra administrator.', 'success');
  } catch (error) {
    setStatus(error.message || 'Registrering mislyktes.', 'error');
  } finally {
    setProcessing(false);
  }
}

async function handleUnlock() {
  if (!currentMemberId) {
    setStatus('Du må registrere nettleseren først.', 'error');
    return;
  }

  setProcessing(true);
  setStatus('Ber om utfordring...', 'neutral');
  try {
    await unlockDoor(currentMemberId);
    setStatus('Døra er låst opp!', 'success');
    setTimeout(() => {
      if (!isProcessing()) {
        setStatus('Klar til å låse opp på nytt.', 'neutral');
      }
    }, 3000);
  } catch (error) {
    setStatus(error.message || 'Klarte ikke å låse opp døra.', 'error');
  } finally {
    setProcessing(false);
  }
}

function handleDelete() {
  deleteRegistration();
  showRegistrationState();
  setStatus('Registrering slettet. Du kan registrere på nytt når som helst.', 'neutral');
}

initialize();
onRegister(handleRegistration);
onUnlock(handleUnlock);
onDelete(handleDelete);
checkRegistrationStatus();

import { getStoredMemberId } from './ttl-crypto.mjs';
import { checkRegistration, deleteRegistration, registerMember } from './registration.mjs';
import { unlockDoor } from './unlock.mjs';
import { getErrorInfo } from './errors.mjs';
import {
  initializeLanguage,
  setLanguage,
  translate,
  translateDocument,
} from './i18n.mjs';
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
  setStatus(translate('status.registrationRequired'), 'neutral');
}

async function checkRegistrationStatus() {
  const memberId = getStoredMemberId();
  if (!memberId) {
    showRegistrationState();
    return;
  }

  currentMemberId = memberId;
  setMemberId(memberId);
  setStatus(translate('status.checkingRegistration'), 'neutral');

  try {
    await checkRegistration(memberId);
    setStatus(translate('status.registrationVerified'), 'success');
    showMemberPanel();
  } catch (error) {
    const errorInfo = error.type
      ? error
      : getErrorInfo(error.message, memberId);
    setStatus(errorInfo.message, 'error');
    if (error.type === 'registration_missing' || error.type === 'keys_missing') {
      showRegistrationPanel();
    } else if (error.type === 'member_expired') {
      showMemberPanel(translate('member.expiredCopy'));
    } else if (error.type === 'member_inactive') {
      showMemberPanel(translate('member.inactiveCopy'));
    } else {
      showMemberPanel();
    }
  }
}

async function handleRegistration() {
  const { name, email } = registrationInput();
  if (!name || !email) {
    setStatus(translate('error.requiredFields'), 'error');
    return;
  }

  setProcessing(true);
  try {
    const member = await registerMember(name, email);
    currentMemberId = member.id;
    setMemberId(member.id);
    clearRegistrationForm();
    showMemberPanel(translate('status.registrationPending'));
    setStatus(translate('status.registrationComplete'), 'success');
  } catch (error) {
    setStatus(error.message || translate('error.registrationFailed'), 'error');
  } finally {
    setProcessing(false);
  }
}

async function handleUnlock() {
  if (!currentMemberId) {
    setStatus(translate('status.registrationRequired'), 'error');
    return;
  }

  setProcessing(true);
  setStatus(translate('status.unlockRequest'), 'neutral');
  try {
    await unlockDoor(currentMemberId);
    setStatus(translate('status.unlocked'), 'success');
    setTimeout(() => {
      if (!isProcessing()) {
        setStatus(translate('status.readyAgain'), 'neutral');
      }
    }, 3000);
  } catch (error) {
    setStatus(error.message || translate('error.unlockFailed'), 'error');
  } finally {
    setProcessing(false);
  }
}

function handleDelete() {
  deleteRegistration();
  showRegistrationState();
  setStatus(translate('status.registrationDeleted'), 'neutral');
}

function changeLanguage(language) {
  setLanguage(language);
  translateDocument();
  checkRegistrationStatus();
}

const language = initializeLanguage();
translateDocument();
initialize();
onRegister(handleRegistration);
onUnlock(handleUnlock);
onDelete(handleDelete);
document.getElementById('language-select').addEventListener('change', (event) => {
  changeLanguage(event.target.value);
});
document.getElementById('language-select').value = language;
checkRegistrationStatus();

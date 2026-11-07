const elements = {
  status: document.getElementById('status-message'),
  registrationPanel: document.getElementById('registration-panel'),
  memberPanel: document.getElementById('member-panel'),
  memberPanelCopy: document.getElementById('member-panel-copy'),
  registeredMemberId: document.getElementById('registered-member-id'),
  nameField: document.getElementById('register-member-name'),
  emailField: document.getElementById('register-member-email'),
  registerButton: document.getElementById('register-browser'),
  unlockButton: document.getElementById('unlock-door'),
  deleteButton: document.getElementById('delete-registration'),
};

let processing = false;

export function setHidden(element, hidden) {
  element.classList.toggle('hidden', hidden);
}

export function setStatus(message, type = 'neutral') {
  console.log(`Status: ${message} (type: ${type})`);
  elements.status.textContent = message;
  elements.status.className = `status-box ${type}`;
}

export function setProcessing(value) {
  processing = value;
  elements.unlockButton.disabled = value;
  elements.registerButton.disabled = value;
  if (value) {
    setStatus(translate('status.waiting'), 'neutral');
  }
}

export function isProcessing() {
  return processing;
}

export function showRegistrationPanel() {
  setHidden(elements.registrationPanel, false);
  setHidden(elements.memberPanel, true);
}

export function showMemberPanel(copy = translate('member.ready')) {
  elements.memberPanelCopy.textContent = copy;
  setHidden(elements.registrationPanel, true);
  setHidden(elements.memberPanel, false);
}

export function setMemberId(memberId) {
  elements.registeredMemberId.textContent = memberId;
}

export function clearRegistrationForm() {
  elements.nameField.value = '';
  elements.emailField.value = '';
}

export function registrationInput() {
  return {
    name: elements.nameField.value.trim(),
    email: elements.emailField.value.trim(),
  };
}

export function onRegister(callback) {
  elements.registerButton.addEventListener('click', callback);
}

export function onUnlock(callback) {
  elements.unlockButton.addEventListener('click', callback);
}

export function onDelete(callback) {
  elements.deleteButton.addEventListener('click', callback);
}

export function initialize() {
  setStatus(translate('status.checking'), 'neutral');
  setHidden(elements.registrationPanel, true);
  setHidden(elements.memberPanel, true);
}
import { translate } from './i18n.mjs';

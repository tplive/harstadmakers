import { translate } from './i18n.mjs';

export function getErrorInfo(backendError, memberId = '') {
  const error = (backendError || '').toLowerCase();
  const registration = memberId ? ` (${memberId})` : '';

  if (error.includes('member not found')) {
    return {
      type: 'registration_missing',
      message: translate('error.registrationMissing', { member: registration }),
    };
  }
  if (error.includes('expired') || error.includes('utløpt')) {
    return {
      type: 'member_expired',
      message: translate('error.memberExpired', { member: registration }),
    };
  }
  if (error.includes('member is inactive') || error.includes('inactive')) {
    return {
      type: 'member_inactive',
      message: translate('error.memberInactive', { member: registration }),
    };
  }
  if (error.includes('private key') || error.includes('not found in browser storage')) {
    return {
      type: 'keys_missing',
      message: translate('error.keysMissing'),
    };
  }
  if (error.includes('signature') || error.includes('verifisering')) {
    return {
      type: 'verification_failed',
      message: translate('error.verificationFailed'),
    };
  }
  if (error.includes('network') || error.includes('fetch') || error.includes('koble til')) {
    return {
      type: 'network_error',
      message: translate('error.network'),
    };
  }
  if (error.includes('challenge') || error.includes('nonce')) {
    return {
      type: 'challenge_error',
      message: backendError || translate('error.challenge'),
    };
  }

  return { type: 'unknown', message: backendError || translate('error.unknown') };
}

export function createAppError(type, message, original = message) {
  return { type, message, original };
}

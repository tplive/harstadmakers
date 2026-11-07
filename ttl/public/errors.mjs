export function getErrorInfo(backendError, memberId = '') {
  const error = (backendError || '').toLowerCase();
  const registration = memberId ? ` (${memberId})` : '';

  if (error.includes('member not found')) {
    return {
      type: 'registration_missing',
      message: `Registreringen din${registration} finnes ikke på serveren. Den kan ha blitt slettet. Vennligst registrer på nytt.`,
    };
  }
  if (error.includes('expired') || error.includes('utløpt')) {
    return {
      type: 'member_expired',
      message: `Tilgangen din${registration} er utløpt og deaktivert. Kontakt administrator for reaktivering.`,
    };
  }
  if (error.includes('member is inactive') || error.includes('inactive')) {
    return {
      type: 'member_inactive',
      message: `Din registrering${registration} er ikke aktivert enda. Vennligst vent på at administrator godkjenner den.`,
    };
  }
  if (error.includes('private key') || error.includes('not found in browser storage')) {
    return {
      type: 'keys_missing',
      message: 'Nøkkeldata mangler. Vennligst registrer denne nettleseren på nytt.',
    };
  }
  if (error.includes('signature') || error.includes('verifisering')) {
    return {
      type: 'verification_failed',
      message: 'Server-verifisering mislyktes. Prøv på nytt.',
    };
  }
  if (error.includes('network') || error.includes('fetch') || error.includes('koble til')) {
    return {
      type: 'network_error',
      message: 'Kunne ikke koble til serveren. Sjekk at du er koblet til riktig Wi-Fi-nettverk.',
    };
  }
  if (error.includes('challenge') || error.includes('nonce')) {
    return {
      type: 'challenge_error',
      message: backendError || 'Utfordringsfeil. Prøv på nytt.',
    };
  }

  return { type: 'unknown', message: backendError || 'Ukjent feil.' };
}

export function createAppError(type, message, original = message) {
  return { type, message, original };
}

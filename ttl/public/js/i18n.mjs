const STORAGE_KEY = 'ttl-language';
const FALLBACK_LANGUAGE = 'en';

export const SUPPORTED_LANGUAGES = {
  en: 'English',
  nb_no: 'Norsk bokmål',
  se: 'Davvisámegiella',
};

// Add a language to an individual key without requiring a complete translation.
export const translations = {
  'page.title': {
    en: 'Harstad Makers',
  },
  'brand.name': {
    en: 'Harstad Makers',
  },
  'hero.eyebrow': {
    en: 'Access to the premises',
    nb_no: 'Tilgang til lokalene',
    se: 'Beassan lokálaide',
  },
  'hero.badge': {
    en: 'Access control',
    nb_no: 'Adgangskontroll',
    se: 'Beassan dárbbašeapmi',
  },
  'hero.heading': {
    en: 'Door access',
    nb_no: 'Dørtilgang',
    se: 'Uksaláhtta',
  },
  'hero.description': {
    en: 'This website is used to unlock the door to Harstad Makers.',
    nb_no: 'Denne nettsiden brukes for å låse opp døra til Harstad Makers.',
    se: 'Dát neahttasiidu geavahuvvo rahpat Harstad Makers-lanja uvssa.',
  },
  'status.checking': {
    en: 'Checking registration status...',
    nb_no: 'Sjekker registreringsstatus...',
  },
  'registration.heading': {
    en: 'Register this browser',
    nb_no: 'Registrer denne nettleseren',
    se: 'Čáli dan fierpmádatloggári',
  },
  'registration.description': {
    en: 'Before you can unlock the door, you must register this device. This registration applies only to this browser.',
    nb_no: 'Før du kan låse opp døra, må du registrere denne enheten. Denne registreringen gjelder kun for denne nettleseren.',
    se: 'Ovdal go sáhtát rahpat uvssa, fertet čáli dán ovttadaga.',
  },
  'form.fullName': {
    en: 'Full name',
    nb_no: 'Fullt navn',
    se: 'Olles namma',
  },
  'form.email': {
    en: 'Email',
    nb_no: 'E-post',
    se: 'E-poasta',
  },
  'form.fullNamePlaceholder': {
    en: 'Jane Maker',
  },
  'form.emailPlaceholder': {
    en: 'jane@harstadmakers.no',
  },
  'registration.button': {
    en: 'Register this browser',
    nb_no: 'Registrer denne nettleseren',
    se: 'Čáli dan fierpmádatloggári',
  },
  'member.heading': {
    en: 'Ready to unlock',
    nb_no: 'Klar til å låse opp',
    se: 'Gearggus rahpat',
  },
  'member.ready': {
    en: 'The browser is registered and ready to use.',
    nb_no: 'Nettleseren er registrert og klar til bruk.',
    se: 'Fierpmádatloggár lea čálihuvvon ja gearggus geavahussii.',
  },
  'member.registeredAs': {
    en: 'Registered as',
    nb_no: 'Registrert som',
    se: 'Čálihuvvon nugo',
  },
  'unlock.button': {
    en: 'Unlock the door',
    nb_no: 'Lås opp døra',
    se: 'Raba uksa',
  },
  'registration.delete': {
    en: 'Delete registration',
    nb_no: 'Slett registrering',
    se: 'Sihko čáliheami',
  },
  'language.label': {
    en: 'Language',
    nb_no: 'Språk',
    se: 'Giella',
  },
  'status.waiting': {
    en: 'Please wait...',
    nb_no: 'Venter...',
  },
  'status.registrationRequired': {
    en: 'You must register this browser first.',
    nb_no: 'Du må registrere denne nettleseren først.',
    se: 'Fertet čáli dán fierpmádatloggári vuos.',
  },
  'status.checkingRegistration': {
    en: 'Checking registration on server...',
    nb_no: 'Sjekker registrering på server...',
  },
  'status.registrationVerified': {
    en: 'Registration verified. Ready to unlock.',
    nb_no: 'Registrering bekreftet. Klar til å låse opp.',
  },
  'status.registrationComplete': {
    en: 'Registration complete! Wait for administrator approval.',
    nb_no: 'Registrering fullført! Vent på godkjenning fra administrator.',
    se: 'Čáliheapmi gárvvui! Vuordde hálddašeaddji dohkkeheami.',
  },
  'status.registrationPending': {
    en: 'Registration is waiting for administrator approval.',
    nb_no: 'Registreringen venter på godkjenning fra administrator.',
    se: 'Čáliheapmi vuordá hálddašeaddji dohkkeheami.',
  },
  'status.registrationDeleted': {
    en: 'Registration deleted. You can register again at any time.',
    nb_no: 'Registrering slettet. Du kan registrere på nytt når som helst.',
  },
  'status.unlockRequest': {
    en: 'Requesting challenge...',
    nb_no: 'Ber om utfordring...',
    se: 'Ozan hástalusa...',
  },
  'status.unlocking': {
    en: 'Unlocking the door...',
    nb_no: 'Låser opp døra...',
  },
  'status.unlocked': {
    en: 'The door is unlocked!',
    nb_no: 'Døra er låst opp!',
    se: 'Uksa lea rabas!',
  },
  'status.readyAgain': {
    en: 'Ready to unlock again.',
    nb_no: 'Klar til å låse opp på nytt.',
  },
  'error.requiredFields': {
    en: 'Full name and email are required.',
    nb_no: 'Fullt navn og e-post må fylles ut.',
    se: 'Olles namma ja e-poasta ferte leat deavddui.',
  },
  'error.registrationFailed': {
    en: 'Registration failed.',
    nb_no: 'Registrering mislyktes.',
    se: 'Čáliheapmi filtii.',
  },
  'error.unlockFailed': {
    en: 'Could not unlock the door.',
    nb_no: 'Klarte ikke å låse opp døra.',
    se: 'Ii sáhttán rahpat uvssa.',
  },
  'error.keysMissing': {
    en: 'Key data is missing. Please register this browser again.',
    nb_no: 'Nøkkeldata mangler. Vennligst registrer denne nettleseren på nytt.',
    se: 'Čoavddadátat váilot. Čáli dán fierpmádatloggári ođđasit.',
  },
  'error.verificationFailed': {
    en: 'Server verification failed. Please try again.',
    nb_no: 'Server-verifisering mislyktes. Prøv på nytt.',
    se: 'Bálvva duođašteapmi filtii. Geahččal ođđasit.',
  },
  'error.registrationMissing': {
    en: 'Your registration{{member}} was not found on the server. It may have been deleted. Please register again.',
    nb_no: 'Registreringen din{{member}} finnes ikke på serveren. Den kan ha blitt slettet. Vennligst registrer på nytt.',
  },
  'error.memberExpired': {
    en: 'Your access{{member}} has expired and is disabled. Contact an administrator to reactivate it.',
    nb_no: 'Tilgangen din{{member}} er utløpt og deaktivert. Kontakt administrator for reaktivering.',
  },
  'error.memberInactive': {
    en: 'Your registration{{member}} has not been activated yet. Please wait for administrator approval.',
    nb_no: 'Din registrering{{member}} er ikke aktivert enda. Vennligst vent på at administrator godkjenner den.',
  },
  'error.network': {
    en: 'Could not connect to the server. Check that you are connected to the correct Wi-Fi network.',
    nb_no: 'Kunne ikke koble til serveren. Sjekk at du er koblet til riktig Wi-Fi-nettverk.',
  },
  'error.challenge': {
    en: 'Challenge error. Please try again.',
    nb_no: 'Utfordringsfeil. Prøv på nytt.',
  },
  'error.unknown': {
    en: 'Unknown error.',
    nb_no: 'Ukjent feil.',
  },
  'error.registrationVerification': {
    en: 'Server verification failed during registration.',
    nb_no: 'Server-verifisering mislyktes under registrering.',
  },
  'member.expiredCopy': {
    en: 'Access has expired and is disabled. Contact an administrator to reactivate it.',
    nb_no: 'Tilgangen er utløpt og deaktivert. Kontakt administrator for reaktivering.',
  },
  'member.inactiveCopy': {
    en: 'Registration is waiting for administrator approval.',
    nb_no: 'Registreringen venter på godkjenning fra administrator.',
    se: 'Čáliheapmi vuordá hálddašeaddji dohkkeheami.',
  },
};

let currentLanguage = FALLBACK_LANGUAGE;

function storage() {
  return globalThis.localStorage;
}

export function getLanguage() {
  return currentLanguage;
}

export function setLanguage(language) {
  currentLanguage = SUPPORTED_LANGUAGES[language] ? language : FALLBACK_LANGUAGE;
  storage()?.setItem(STORAGE_KEY, currentLanguage);
  return currentLanguage;
}

export function initializeLanguage() {
  const stored = storage()?.getItem(STORAGE_KEY);
  currentLanguage = SUPPORTED_LANGUAGES[stored] ? stored : FALLBACK_LANGUAGE;
  return currentLanguage;
}

export function translate(key, variables = {}) {
  const value = translations[key]?.[currentLanguage]
    ?? translations[key]?.[FALLBACK_LANGUAGE]
    ?? key;
  return value.replace(/\{\{(\w+)\}\}/g, (_, name) => variables[name] ?? '');
}

export function translateDocument(root = document) {
  root.querySelectorAll('[data-i18n]').forEach((element) => {
    element.textContent = translate(element.dataset.i18n);
  });
  root.querySelectorAll('[data-i18n-placeholder]').forEach((element) => {
    element.placeholder = translate(element.dataset.i18nPlaceholder);
  });
  root.querySelectorAll('[data-i18n-aria-label]').forEach((element) => {
    element.setAttribute('aria-label', translate(element.dataset.i18nAriaLabel));
  });
  root.documentElement?.setAttribute('lang', currentLanguage);
}

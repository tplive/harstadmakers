// Translations for Harstad Makers
// Norwegian (Bokmål)

const translations = {
    // General
    siteTitle: "Harstad Makers",
    communityWorkspace: "Felles arbeidsområde",
    
    // Navigation
    navOverview: "Oversikt",
    navDoorAccess: "Dørtilgang",
    navApi: "API",
    
    // Hero section
    heroHeadline: "Tilgang Til Lokalene",
    heroDescription: "Denne nettsiden kan du bruke til å låse opp døra!",
    
    // Debug actions
    actionLoadMembers: "Hent medlemmer",
    actionRegisterMember: "Registrer denne nettleseren",
    actionRequestChallenge: "Be om dør-challenge",
    
    // Status
    accessStatus: "Tilgangsstatus",
    statusWaiting: "Venter på innsjekk.",
    statusWaitingShort: "Venter på innsjekk.",
    
    // Register panel
    registerPanelTitle: "Registrer denne nettleseren",
    registerPanelDescription: "Denne visningen brukes for vanlige medlemmer før nettleseren er registrert.",
    labelFullName: "Fullt navn",
    labelEmail: "E-post",
    placeholderFullName: "Jane Maker",
    placeholderEmail: "jane@harstadmakers.no",
    buttonRegisterBrowser: "Registrer denne nettleseren",
    
    // Member panel
    memberPanelTitle: "Klar til å låse opp",
    memberPanelDescription: "Nettleseren er registrert. Trykk for å starte hele opplåsingsflyten.",
    labelRegisteredMemberId: "Registrert medlems-ID",
    buttonUnlockDoorQuick: "Lås opp døra",
    buttonDeleteRegistration: "Slett denne nettleserregistreringen",
    
    // Debug panel
    debugPanelTitle: "Opplåsing av dør i X enkle steg...",
    labelMemberId: "Medlems ID",
    labelDebugMemberName: "Fullt navn",
    labelDebugMemberEmail: "E-post",
    labelChallengeId: "Challenge ID",
    placeholderChallengeId: "Returneres fra backend",
    labelDecryptedNonce: "Dekryptert nonce fra mobilappen",
    placeholderDecryptedNonce: "Appen dekrypterte challenge-bytene her",
    buttonUnlockDoor: "Lås opp døra",
    
    // Challenge data
    challengeDataTitle: "Challenge-data",
    challengeNotReceived: "Ikke mottatt challenge enda.",
    
    // API documentation
    apiTitle: "Backend API",
    
    // Placeholders
    placeholderMemberId: "demo-member",
};

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = translations;
}

# TODO

## Tilgangskontroll

- [x] Implementere client-nonce-flyten som er beskrevet i `README.md`.
  - Nettleseren skal generere en client-nonce.
  - Nettleseren skal signere client-nonce med sin Ed25519-private nøkkel.
  - Serveren skal verifisere signaturen med registrert offentlig nøkkel.
- [x] Implementere utløp og kontroll av challenge etter fem minutter.
- [x] Sikre at serveren genererer en ny, engangs challenge for hvert opplåsingsforsøk.
- [ ] Avvise gjenbruk av en challenge etter en vellykket opplåsing.
- [ ] Verifisere challenge- og nonce-reglene med automatiserte tester.
- [x] Implementere relé- og dørlåsmodulen.
  - Bruk en mock av relé og dørlås inntil fysisk maskinvareintegrasjon er klar.
  - Mocken skal kunne brukes til å verifisere at en godkjent opplåsing aktiverer dørlåsen i riktig tidsrom.
- [x] Fjerne `debug_member_id` fullstendig.
- [x] Fjerne `is_debug` feltet fra lagrede metadata.
- [x] Fjerne API-endepunktet for å godkjenne medlemmer.
  - Aktivering skal fortsatt gjøres manuelt ved redigering av `backend/data/members.json`.
  - Tilgangsansvarlig logger inn på serveren via SSH, utenfor tilgangskontrollsystemet.
  - Tilgangsansvarlig kontrollerer siste siffer i UUID manuelt sammen med brukeren før `active` settes til `true`.
- [ ] Forbedre nettverksbinding og tilgangsbegrensning før produksjonssetting.
  - `0.0.0.0:3000` er akseptabelt under utvikling, men må begrenses eller sikres i produksjon.
  - Vurder også CORS-konfigurasjonen før produksjonssetting.
- [ ] Begrense tjenesten til det private Wi-Fi-nettverket.
- [ ] Begrense API-tilgang til betrodde mobile enheter på nettverket.
- [ ] Fjerne wildcard CORS og tillate bare nødvendige origins dersom nettlesertilgang beholdes.
- [ ] Vurdere direkte app-til-server-trafikk fremfor generell nettlesertilgang.
- [ ] Legge til ratebegrensning for registrering, challenge-forespørsler og opplåsing.
- [ ] Bygge applikasjonen for Raspberry Pi.
- [ ] En Hall-sensor på døren kobles til en GPIO pinne på Rasberry Pi. Den trigger når døren er åpen og lukket, slik at vi kan lukke låsen når døren går igjen. Og, vi kan merke om døren åpnes uten at dørlåsen er trigget.

## Autentisering og autorisasjon

- [ ] Kreve besittelse av den registrerte private nøkkelen for alle medlemsoperasjoner.
- [ ] Knytte hver medlemskonto til én offentlig nøkkel lagret i `members.json`.
- [ ] Avvise alle opplåsingsforsøk som ikke beviser besittelse av tilhørende private nøkkel.
- [ ] Verifisere at hver challenge er knyttet til medlemmet som sender opplåsingsforespørselen.
- [ ] Kontrollere at returnert nonce er identisk med den lagrede challenge-nonce-en.

## Datahåndtering

- [ ] Beskytte `members.json` og `door_events.json` på Raspberry Pi-en.
- [ ] Begrense filtillatelser slik at bare tjenesteoperatøren kan redigere filene.
- [ ] Ta sikkerhetskopi av `members.json` før manuelle endringer.
- [ ] Validere JSON etter hver manuell endring.
- [ ] Føre endringslogg for manuelle medlems- og debug-oppdateringer.

## Logging og hemmeligheter

- [ ] Slutte å skrive private nøkler eller annet sensitivt registreringsmateriale til stdout.
- [ ] Logge registrering, aktivering, challenge-forespørsler og opplåsingsforsøk.
- [ ] Inkludere tidsstempel og målrettet medlems-ID i revisjonslogger.
- [ ] Overvåke mislykkede opplåsinger og gjentatte challenge-forespørsler.
- [ ] Oppbevare SSH-nøkler og annen serveridentitet kun på Raspberry Pi-en.

## Vertssikring og fysisk sikkerhet

- [ ] Dokumentere at backend-serveren er betrodd og fysisk kontrollerer dørlåsen.
- [ ] Sikre operativsystemet på Raspberry Pi-en.
- [ ] Installere sikkerhetsoppdateringer for operativsystem og runtime regelmessig.
- [ ] Begrense shell-tilgang til betrodde tilgangsansvarlige.
- [ ] Deaktivere unødvendige tjenester på Raspberry Pi-en.
- [ ] Sørge for at styringsbanen til dørlåsen er fysisk koblet til den betrodde serveren.
- [ ] Sørge for en manuell fysisk overstyring av dørlåsen.

## Gjenoppretting og hendelseshåndtering

- [ ] Definere hvordan en kompromittert medlemsnøkkel tilbakekalles.
- [ ] Definere hvordan all opplåsing deaktiveres raskt ved en sikkerhetshendelse.
- [ ] Definere hvordan `members.json` gjenopprettes fra sikkerhetskopi.
- [ ] Teste hele den manuelle aktiverings- og tilbakekallingsprosessen.
- [ ] Gjennomgå revisjonsloggen etter hver sikkerhetshendelse.

## Testing

- [ ] Gjøre frontendtestene kjørbare i utviklingsmiljøet.
  - `public/ttl-crypto.test.mjs` skal kunne kjøres med Node.js.
  - Dokumenter eventuell nødvendig Node.js-versjon og testkommando.
- [ ] Utvide testene for client-nonce-flyten.
- [ ] Utvide testene for challenge-utløp.
- [ ] Utvide testene for mocken av relé og dørlås.

## Infrastruktur og dokumentasjon

- [ ] Verifisere og dokumentere produksjonsoppsettet for Nginx, TLS/Let's Encrypt, lokal DNS/DHCP og Wi-Fi-begrensning.
- [ ] Beholde `README.md` som målbeskrivelse for systemet og oppdatere den først når de tilsvarende funksjonene er implementert og verifisert.

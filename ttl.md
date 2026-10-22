# Harstad Makers TTL (Tilgang Til Lokalene)

Dette prosjektet er en lokal tilgangskontroll for å komme inn i våre lokaler. Tilgangskontrollen vil bli koblet opp til medlemsregisteret, og kun medlemmer vil kunne få tilgang.

Den endelige designen er bevisst enkel:

- Medlemmer må registrere seg før de kan bruke tilgangskontrollen. Dette gjøres èn gang, men må gjentas hvis man bytter telefon eller rensker nettleseren på telefonen, da tilgangskoden lagres der. En bruker kan ha flere nettlesere godkjent. Feks en mobil og en på sin pc.
- Vi har en liten datamaskin (RPI) på innsiden som fysisk styrer dørlåsen. Denne kjører også nettside og API for dørkontrollen.
- RPI holder en enkel nettside som gjøres tilgjengelig på det private Wi‑Fi-nettverket.
- Medlemmer som vil inn, åpner nettstedet fra mobilen sin mens de er koblet til vår wifi, fra utsiden av døren.
- Registreringen sørger for at RPI lagrer registrerte tilgangsnøkkel for medlemmer.
- Nettleseren beviser at den har den motsvarende tilgangsnøkkelen ved å kryptere en utfordring fra RPI
- RPI validerer resultatet lokalt og aktiverer dørlåsen dersom alt stemmer.

## Arkitektur

- Frontend: statisk HTML, CSS og JavaScript som serveres lokalt av Raspberry Pi-en over det private Wi‑Fi-nettverket.
- Backend: Rust + Axum-tjeneste for lagring av medlemmer, generering av utfordringer og autorisasjonskontroll.
- Persistens: JSON-baserte logger og medlemsdata i backend-data-mappen.
- Tilgangsflyt:
  - medlemmet kobler til trådløsnettet og åpner nettsiden.
  - medlemmet trykker på "Lås opp døren"
  - mobilnettsiden ber RPI om en challenge
  - RPI genererer en "nonce" som den krypterer med medlemmets registrerte nøkkel
  - mobilen (nettsiden) dekrypterer med medlemmets private nøkkel (fra mobilen)
  - backend validerer "nonce" før låsfunksjonen blir tillatt.
- Dørmodul: RPI innenfor døren trigger 12V relè som kobler inn dørlåsen i x antall sekunder.

## Sikkerhetsmodell

Nettleser på mobilen genererer privat og offentlig nøkkel og lagrer disse på telefonen. Nettsiden sender medlemmets offentlige nøkkel til RPI som lagrer denne sammen med navn og metadata.

Ved aktivering, genererer backend en engangsnonce, krypterer den med medlemmets offentlige nøkkel og returnerer kryptert payload til klienten. Mobilnettleseren dekrypterer utfordringen med den private nøkkelen og sender klarteksten tilbake for verifisering. Når klarteksten matcher lagret nonce og medlemmet er på riktig Wi‑Fi-SSID, godkjenner backend døråpning, og RPI trigger dørlåsen.

Dette holder systemet fullt lokalt og unngår å eksponere tilgangs-API-et på det offentlige internett.

Trafikk mellom nettleser på mobil og RPI er HTTPS.

Backend forventer en eksisterende Ed25519 SSH-nøkkel i `~/.ssh/id_ed25519` og en offentlig nøkkel i `~/.ssh/id_ed25519.pub`. Hvis den offentlige nøkkelen i stedet ligger som `~/.ssh/if_ed25519.pub`, aksepteres også det navnet.

## Repo-struktur

- `public/` — statisk nettsted som serveres av den lokale Raspberry Pi-en.
- `backend/` — Rust API-tjeneste
- `architecture-diagrams.md` — arkitektur- og flytdiagrammer

## Hurtig start

Start Rust-backend:

```bash
cd backend
cargo run
```

Backend lytter på port 3000 og lagrer medlemsdata i `backend/data/`.

## Host frontend lokalt for testing

For å teste nettsiden lokalt på samme private Wi‑Fi-nettverk, brukes frontend direkte fra prosjektet uten Netlify eller offentlig hosting.

### 1) Start backend først

```bash
cd backend
cargo run
```

Dette starter Rust-API-et på `http://localhost:3000` på Raspberry Pi-en eller den lokale tjeneren.

### 2) Start en lokal webserver for frontend

Fra repo-roten kan du starte en enkel lokal webserver:

```bash
cd public
python3 -m http.server 8080
```

Dette gjør nettstedet tilgjengelig på:

```text
http://localhost:8080
```

Hvis du tester fra en telefon på samme Wi‑Fi-nettverk, bruk i stedet Raspberry Pi-en sin lokale IP-adresse:

```text
http://192.168.1.50:8080
```

Erstatt `192.168.1.50` med den faktiske IP-adressen til Pi-en.

### 3) Verifiser at frontend peker riktig mot backend

I `public/app.js` ligger `API_BASE`-konstanten. Den må peke mot den lokale backend-tjeneren, for eksempel:

```js
const API_BASE = 'http://192.168.1.50:3000';
```

Hvis frontend og backend ligger på samme maskin, kan også `http://localhost:3000` brukes.

### 4) Test fra mobilen

1. Koble telefonen til samme private Wi‑Fi-nettverk som Raspberry Pi-en.
2. Åpne nettadressen til Pi-en i nettleseren, for eksempel:
   `http://192.168.1.50:8080`
3. Klikk på "Request door challenge" eller tilsvarende testknapp.
4. Verifiser at frontend får respons fra backend.
5. Utfør oppgaveflyten med detaljerte testverdier fra demo-medlemmet.

### 5) Hvis du vil bruke samme lokale adresse på flere enheter

For lettere testing i samme nettverk kan du også bruke en kjent hostname eller Pi-en sin lokale `.local`-adresse, for eksempel:

```text
http://raspberrypi.local:8080
```

## HTTPS / Let's Encrypt for frontend og backend

Let's Encrypt verifiserer at man eier domenet. Det inneholder ikke IP-adressen, og derfor kan man få sertifikater som fungerer helt fint på innsiden av ruteren, så lenge navneoppslag benytter lokal DNS som peker mot LAN ip-adresser.

Det betyr at:

- domenet må kunne verifiseres av Let's Encrypt over internett, utgående trafikk
- RPI står på LAN bak ruteren, privat IP adresse.
- RPI trenger ikke publiseres som en offentlig IP-tjeneste for alle, så lenge ruteren kan sende ACME-kallet videre og domenet er korrekt konfiguert.

### Enkel oppsett for dette prosjektet

Vi bruker to DNS-navn:

- `ttl.harstadmakers.no` → frontend
- `ttl-api.harstadmakers.no` → backend

I ruteren peker vi disse navnene til de lokale IP-adressene til RPI, for eksempel:

```text
ttl.harstadmakers.no      -> 192.168.1.50
ttl-api.harstadmakers.no  -> 192.168.1.50
```

Dette gjør at medlemmer på det lokale nettverket kan gå til:

```text
https://ttl.harstadmakers.no
https://ttl-api.harstadmakers.no
```

uten at disse navnene trenger å bli publisert i public DNS.

### Nginx som TLS-terminator

På RPI installerer du Nginx og konfigurerer den som reverse proxy til frontend og backend.

Det eneste som skal være åpent mot internett er port 80 for Let's Encrypt HTTP-01 challenge. Alt annet, inkludert 443, 3000, 8080 og enhver intern tjeneste, skal holdes bak ruteren/firewallen og være utilgjengelig fra internett.

Eksempel:

```nginx
# Port 80 er kun åpent for ACME challenge.
# Ingen andre tjenester skal være publisert direkte mot internett.
server {
    listen 80;
    server_name ttl.harstadmakers.no;

    location /.well-known/acme-challenge/ {
        root /var/www/letsencrypt;
    }

    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name ttl.harstadmakers.no;

    ssl_certificate /etc/letsencrypt/live/ttl.harstadmakers.no/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/ttl.harstadmakers.no/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
    }
}

server {
    listen 80;
    server_name ttl-api.harstadmakers.no;

    location /.well-known/acme-challenge/ {
        root /var/www/letsencrypt;
    }

    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name ttl-api.harstadmakers.no;

    ssl_certificate /etc/letsencrypt/live/ttl-api.harstadmakers.no/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/ttl-api.harstadmakers.no/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
    }
}
```

### ACME / certbot

Installer Certbot:

```bash
sudo apt update
sudo apt install -y certbot python3-certbot-nginx
```

Hent sertifikatene:

```bash
sudo certbot --nginx -d ttl.harstadmakers.no -d ttl-api.harstadmakers.no
```

Dette fungerer fordi Let’s Encrypt kan verifisere domenenavnet via DNS og HTTP-01, selv om tjeneren faktisk ligger bak NAT og bruker en lokal IP i hjemmenettet.

### Viktige forhold

- Navnet må være registrert i offentlig DNS og verifiserbart for ACME.
- Bare port 80 skal være åpen for HTTP-01 challenge-tilgang fra internett.
- Alle andre porter må være lukket bak ruteren eller firewallen.
- IP-adressen bak domenet trenger ikke være offentlig for alle.
- Ruteren kan peke domenet til den lokale Pi-en.
- Frontend og backend kan fortsatt være private i praksis, siden de kun er tilgjengelige via det lokale nettverket og ruteren.

### Praktisk anbefaling

For dette prosjektet er den enkleste og mest robuste modellen:

- bruker ekte DNS-navn for frontend og API
- bruker offentlig ACME for sertifikater
- peker navnet i ruteren til lokale IP adresser på RPI.
- holder tjenestene lokale, men med korrekt HTTPS og automatisk fornyelse

Dette oppfyller både behovet for ekte TLS og ønsket om at systemet skal være lokalt styrt og ikke publisert som en offentlig, åpen tjeneste.

## API-eksempler

Helsetest:

```bash
curl http://localhost:3000/health
```

List medlemmer:

```bash
curl http://localhost:3000/api/members
```

Be om en challenge:

```bash
curl -X POST http://localhost:3000/api/challenge \
  -H 'Content-Type: application/json' \
  -d '{"member_id":"demo-member","server_challenge":"<browser-generated-nonce>"}'
```

Registrer en nettleser og lær serverens offentlige nøkkel:

```bash
curl -X POST http://localhost:3000/api/members \
  -H 'Content-Type: application/json' \
  -d '{
    "name":"Jane Maker",
    "email":"jane@harstadmakers.no",
    "public_key_jwk": {"kty":"RSA","n":"<modulus>","e":"AQAB"},
    "server_challenge":"<browser-generated-nonce>"
  }'
```

Lås opp døren etter at mobilnettleseren har verifisert server-signaturen og dekryptert utfordringen:

```bash
curl -X POST http://localhost:3000/api/door/unlock \
  -H 'Content-Type: application/json' \
  -d '{
    "member_id":"demo-member",
    "challenge_id":"<challenge-id>",
    "decrypted_nonce":"<decrypted-nonce>"
  }'
```

## Deployeringsmodell

Denne løsningen er uttrykkelig designet for å kjøre kun på det private lokale nettverket.

- Nettsiden hostes på RPI.
- Dørkontrolleren er lokalt på stedet. Aktiveres over RPI sine GPIO porter.
- Medlemsnettsiden og API-et antar lokal nettverkstilgang. Er du ikke på makerspacets wifi, kommer du normalt heller ikke fram til tjenesten.
- API-et forblir utenfor offentlig internett.

## Gjensidig verifisering

- Ved registrering sender nettleseren sin RSA offentlige nøkkel og en egen challenge til API-et.
- API-et signerer denne challengen med serverens Ed25519 SSH-nøkkel og returnerer signaturen sammen med serverens offentlige nøkkel.
- Nettleseren lagrer serverens offentlige nøkkel og bruker den videre for å verifisere at challenge-svaret faktisk kommer fra samme server.
- Ved hver døråpning sendes en ny browser-generert challenge til `/api/challenge`, og nettleseren verifiserer server-signaturen før den stoler på nonce-en som skal dekrypteres.

## Merknader for RPI

Backend er bevisst generell. Den lokale nettverksmodulen kan eksponere en funksjon som `unlock_door` og kalle GPIO-reléet eller låseaktuatoren når API-et godkjenner en challenge. Dette holder tilgangskontroll-logikken lokal og separert fra fysisk dørutstyr.

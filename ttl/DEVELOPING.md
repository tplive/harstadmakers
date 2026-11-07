# Utvikling av løsningen

## Repo-struktur

- `public/` — statisk nettsted som leveres av serveren.
- `backend/` — Rust API-tjeneste
- `architecture-diagrams.md` — arkitektur- og flytdiagrammer

## Kjøre løsningen lokalt

### 1. Database

Backend bruker [Pocketbase](https://pocketbase.io) for lagring av data.

Standard schema for databasen importeres (i Pocketbase admin gui) fra `ttl/backend/pb_schema.json`.

En bruker i users collection har tilgang til å lese, opprette og oppdatere (men ikke slette) poster i members. Samme bruker har tilgang til å lese og opprette (men ikke oppdatere eller slette) poster i door_events.

For å konfigurere ttl-api backend for å bruke denne konfigurasjonen, bruke Impersonate user og lagre tokenet som genereres i configuration.json: `"pocketbase_token"`.

Ved registrering av member vil ID autogenereres av PocketBase og returneres til nettleseren og brukes videre som `member_id` i API-forespørsler. 

I JSON-filene som opprettes når man ikke bruker database heter `member_id`. Ved lagring av dørhendelser opprettes `id` automatisk, og PocketBase sin tildelte ID brukes som `DoorEvent.id`.

Ved oppstart kontrollerer backend at begge collections finnes og at alle feltene over har riktig PocketBase-type. Manglende collections eller feil felttyper stopper oppstart med en konkret feilmelding.

Når `POCKETBASE_URL` er satt, feiler backend dersom databasen ikke er tilgjengelig eller svarer med en feil; den faller ikke tilbake til JSON-filer.

### 2. Backend
Start backend med

```bash
cd backend
cargo run -- -c configuration.json
```

Backend leser som standard `./configuration.json`. En annen fil kan angis med
`-c` eller `--config`:

```bash
cargo run -- --config /path/to/configuration.json
```

The development configuration serves the API over HTTP on `localhost:3000`.
TLS remains enabled by default and is enabled in the production example
configuration. Set `tls_enabled` to `true` or `false` in the configuration
file to choose the transport.

Konfigurasjonen styrer serverens Ed25519-nøkkelstier, challenge- og
medlemsutløp, PocketBase-tilkobling, datakatalog, bind-adresse og GPIO:

- `relay_gpio_pin` er BCM-pinnen som driver reléet og starter lav.
- `hall_sensor_gpio_pin` er BCM-pinnen for hall-sensoren. Den konfigureres med
  intern pull-up og leses lav når døren er åpen.
- `unlock_duration_ms` angir hvor lenge reléet holdes høyt ved opplåsing.
- `gpio_enabled` må settes til `true` på Raspberry Pi for å aktivere GPIO.
  Den bør være `false` på utviklingsmaskiner som ikke har Raspberry Pi-maskinvare.

Raspberry Pi GPIO-signalet er 3,3 V (ikke 5 V) og er ikke 5 V-tolerant.
Relémodulen må derfor ha en 3,3 V-kompatibel inngang eller egnet nivåtilpasning.
PocketBase brukes når `pocketbase_url` ikke er tom; ellers brukes JSON-filer i
`data_dir`.

Dette starter Rust-API-et over HTTP eller HTTPS, avhengig av `tls_enabled`, på
konfigurasjonens `bind_address`, som standardproduksjonskonfigurasjonen setter
til `0.0.0.0:4433`. Sertifikat og privat nøkkel angis med
`tls_certificate_path` og `tls_private_key_path` i `configuration.json`.

### 3. Frontend

Start en lokal webserver for frontend:

```bash
deno run --allow-net --allow-read jsr:@std/http/file-server \
  --cert /path/to/fullchain.pem \
  --key /path/to/privkey.pem \
  --port 8088 public
```

Port 443 krever normalt root eller `CAP_NET_BIND_SERVICE` på Linux. Kommandoen
over kjører bare Deno med root-rettigheter; stopp den med Ctrl-C. Dette gjør
nettstedet tilgjengelig på `https://ttl.harstadmakers.no` når DNS-navnet peker
til denne maskinen. API-et bruker `https://ttl-api.harstadmakers.no:4433`.

## API-eksempler

Helsetest:

```bash
curl -k https://ttl-api.harstadmakers.no:4433/health
```

Be om en challenge:

```bash
curl -k -X POST https://ttl-api.harstadmakers.no:4433/api/challenge \
  -H 'Content-Type: application/json' \
  -d '{"member_id":"demo-member","server_challenge":"<browser-generated-nonce>","client_nonce":"<client-nonce>","client_nonce_signature":"<Ed25519-signature>"}'
```

Registrer en nettleser og lær serverens offentlige nøkkel:

```bash
curl -k -X POST https://ttl-api.harstadmakers.no:4433/api/members \
  -H 'Content-Type: application/json' \
  -d '{
    "name":"Jane Maker",
    "email":"jane@harstadmakers.no",
    "public_key_jwk": {"kty":"OKP","crv":"Ed25519","x":"<public-key-bytes>"},
    "server_challenge":"<browser-generated-nonce>"
  }'
```

The browser signs a fresh client nonce with its Ed25519 private key for every
challenge request. The backend verifies `client_nonce_signature` against the
registered public key and returns the client nonce in `client_nonce`. The
browser also signs the returned `server_nonce`; that signature is sent to the
unlock endpoint as `server_nonce_signature`.

Lås opp døren etter at mobilnettleseren har verifisert server-signaturen og signert server-nonce:

```bash
curl -k -X POST https://ttl-api.harstadmakers.no:4433/api/door/unlock \
  -H 'Content-Type: application/json' \
  -d '{
    "member_id":"demo-member",
    "challenge_id":"<challenge-id>",
    "server_nonce_signature":"<Ed25519-signature>"
  }'
```

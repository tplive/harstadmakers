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

Konfigurasjonen styrer serverens Ed25519-nøkkelstier, challenge- og
medlemsutløp, PocketBase-tilkobling, datakatalog og bind-adresse.
PocketBase brukes når `pocketbase_url` ikke er tom; ellers brukes JSON-filer i
`data_dir`.

Dette starter Rust-API-et over HTTPS på konfigurasjonens `bind_address`, som
standardproduksjonskonfigurasjonen setter til `0.0.0.0:4433`. Sertifikat og privat nøkkel angis med
`tls_certificate_path` og `tls_private_key_path` i `configuration.json`.

### 3. Frontend

Start en lokal webserver for frontend:

```bash
deno run --allow-net --allow-read jsr:@std/http/file-server \
  --cert /path/to/fullchain.pem \
  --key /path/to/privkey.pem \
  --port 8088 public
```

Dette gjør nettstedet tilgjengelig på `https://localhost:8088`. I produksjon
bruker frontend-tjenesten `https://ttl.harstadmakers.no` på port 443, mens
API-et bruker `https://ttl-api.harstadmakers.no:4433`.

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
  -d '{"member_id":"demo-member","server_challenge":"<browser-generated-nonce>","client_nonce":"<client-nonce>","client_nonce_signature":"<Ed25519-signature>"}'
```

Registrer en nettleser og lær serverens offentlige nøkkel:

```bash
curl -X POST http://localhost:3000/api/members \
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
curl -X POST http://localhost:3000/api/door/unlock \
  -H 'Content-Type: application/json' \
  -d '{
    "member_id":"demo-member",
    "challenge_id":"<challenge-id>",
    "server_nonce_signature":"<Ed25519-signature>"
  }'
```

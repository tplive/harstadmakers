# Utvikling av løsningen

## Repo-struktur

- `public/` — statisk nettsted som leveres av serveren.
- `backend/` — Rust API-tjeneste
- `architecture-diagrams.md` — arkitektur- og flytdiagrammer

## Kjøre løsningen lokalt

### 1. Database

Backend kan enten lagre medlemsdata i `data/members.json` og hendelser i `data/door_events.json`, men man kan også bruke [Pocketbase](https://pocketbase.io). Hvis du ikke vil bruke PB er det bare å hoppe over dette steget. 

For PocketBase setter du:

```bash
export POCKETBASE_URL=http://127.0.0.1:8090
export POCKETBASE_TOKEN='<server-side PocketBase token>'
```

Collections i Pocketbase heter default `members` og `door_events`. Dersom man absolutt må kalle dem noe annet, kan man sette miljøvariablene `POCKETBASE_MEMBERS_COLLECTION` og `POCKETBASE_EVENTS_COLLECTION` til noe annet.

Men det enkleste er å bruke standard schema. Dette er eksportert og kan importeres `fra ttl/backend/pb_schema.json`.

Backendens token må ha tilgang til å lese, opprette, oppdatere og slette poster.

Ved registrering vil record i `members` opprettes uten `id` - ID'en autogenereres av PocketBase, returneres til nettleseren og brukes videre som `member_id` i API-forespørsler. 

I JSON-filene som opprettes når man ikke bruker database heter `member_id`. Ved lagring av dørhendelser opprettes `id` automatisk, og PocketBase sin tildelte ID brukes som `DoorEvent.id`.

Ved oppstart kontrollerer backend at begge collections finnes og at alle feltene over har riktig PocketBase-type. Manglende collections eller feil felttyper stopper oppstart med en konkret feilmelding.

Når `POCKETBASE_URL` er satt, feiler backend dersom databasen ikke er tilgjengelig eller svarer med en feil; den faller ikke tilbake til JSON-filer.

### 2. Backend
Start backend med

```bash
cd backend
cargo run
```

Dette starter Rust-API-et på `http://localhost:3000` på den lokale tjeneren.

### 3. Frontend

Start en lokal webserver for frontend:

```bash
cd public
python3 -m http.server 8088
```

Dette gjør nettstedet tilgjengelig på `http://localhost:8088`

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

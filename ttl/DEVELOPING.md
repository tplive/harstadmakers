# Utvikling av løsningen

## Repo-struktur

- `public/` — statisk nettsted som leveres av serveren.
- `backend/` — Rust API-tjeneste
- `architecture-diagrams.md` — arkitektur- og flytdiagrammer

## Kjøre løsningen lokalt

1) Start backend først

```bash
cd backend
cargo run
```

Dette starter Rust-API-et på `http://localhost:3000` på den lokale tjeneren.

### Persistens

Uten ekstra miljøvariabler bruker backend filene `data/members.json` og
`data/door_events.json`. For PocketBase setter du:

```bash
export POCKETBASE_URL=http://127.0.0.1:8090
export POCKETBASE_TOKEN='<server-side PocketBase token>'
```

Valgfritt kan samlingsnavnene endres med `POCKETBASE_MEMBERS_COLLECTION` og
`POCKETBASE_EVENTS_COLLECTION`. Standardverdiene er `members` og `door_events`.

Opprett samlingene i PocketBase med disse feltene:

- `members`: `member_id` (text, unique), `name` (text), `email` (text),
  `public_key_pem` (text), `active` (bool), `created_at` (text),
  `last_used` (text, optional)
- `door_events`: `event_id` (text, unique), `member_id` (text), `action` (text),
  `timestamp` (text)

Backendens token må ha tilgang til å lese, opprette, oppdatere og slette poster.
Når `POCKETBASE_URL` er satt, feiler backend eksplisitt dersom PocketBase ikke
kan nås eller svarer med en feil; den faller ikke tilbake til JSON-filer eller
demo-medlem.

2) Start en lokal webserver for frontend

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

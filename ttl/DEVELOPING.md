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

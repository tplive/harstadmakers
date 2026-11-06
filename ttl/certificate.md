# Utstede sertifikat for TTL

* Vi benytter [Domeneshop](https://domene.shop) som DNS provider for vårt domene.
* Vi benytter [acme.sh](https://github.com/acmesh-official/acme.sh) som automatiseringsverktøy.

Acme.sh [kan installeres til valgfri katalog](https://github.com/acmesh-official/acme.sh/wiki/How-to-install#4-advanced-installation) med `--home` parameteret for å spesifisere katalog. Man kan også sette separate kataloger for konfigurasjon og sertifikater med  `--config-home` og `--cert-home`.
Install the tool with:
```bash
git clone --depth 1 https://github.com/acmesh-official/acme.sh.git
cd acme.sh
./acme.sh --install  \
--home /opt/acme.sh \ 
--config-home /opt/acme.sh \
--cert-home  /srv/certs \
--accountemail  "my@example.com"
```

1. Logg inn på [Domeneshop](https://domene.shop) med din konto.
2. Følg instruksjonene: [Use Domeneshop DNS API to automatically issue a cert](https://github.com/acmesh-official/acme.sh/wiki/dnsapi2#85-use-domeneshop-dns-api-to-automatically-issue-a-cert)
3. "You'll have to get a Domeneshop API key and secret (<https://api.domeneshop.no/docs/>)."
4. "To generate credentials, visit this page after logging in to the control panel on our website: <https://www.domeneshop.no/admin?view=api>"
5. Noter token, secret og timestamp for utløp.
6. For manuelt å utstede sertifikat, kjør:

```bash
sudo groupadd --system ttl-certs
sudo usermod --append --groups ttl-certs svc-ttl-api
sudo usermod --append --groups ttl-certs svc-ttl-frontend

sudo install --directory --owner=root --group=ttl-certs --mode=0750 \
  /srv/ttl-certs
```

The service users must be restarted, or a new login session must be started,
before their supplementary `ttl-certs` group membership is visible.

## Install acme.sh under `/srv`

Install acme.sh as root so its renewal timer/cron job can write to
`/srv/.acme.sh` and install the renewed certificate files:

```bash
sudo mkdir -p /srv/.acme.sh
sudo chown root:root /srv/.acme.sh
sudo chmod 700 /srv/.acme.sh

curl https://get.acme.sh | sudo sh -s email=my@example.com \
  --home /srv/.acme.sh
```

Confirm that the installation uses the requested home directory:

```bash
sudo /srv/.acme.sh/acme.sh --home /srv/.acme.sh --version
sudo /srv/.acme.sh/acme.sh --home /srv/.acme.sh --list
```

The DNS API credentials are private. If they are stored as a local JSON file
for the DNS API helper, keep that file inside the acme.sh directory so the
service users cannot read it:

```bash
sudo install --owner=root --group=root --mode=0600 \
  /path/to/domeneshop_api.json /srv/.acme.sh/domeneshop_api.json
```

## Issue the certificate

Log in to Domeneshop and follow the
[Domeneshop DNS API instructions](https://github.com/acme.sh/acme.sh/wiki/dnsapi2#85-use-domeneshop-dns-api-to-automatically-issue-a-cert).
The API token and secret are available from
<https://www.domeneshop.no/admin?view=api>.

For a manual issuance, export the credentials only in the root shell:

```bash
sudo -i
export DOMENESHOP_Token=your_token_here
export DOMENESHOP_Secret=your_secret_here

/srv/.acme.sh/acme.sh --home /srv/.acme.sh \
  --issue --dns dns_domeneshop \
  -d ttl.harstadmakers.no \
  -d ttl-api.harstadmakers.no
```

7. Vi beholder data lokalt på TTL serveren. I admin brukerens hjemmekatalog; lag `domeneshop_api.json`:

## Install renewed certificates into `/srv/ttl-certs`

Use `--install-cert` rather than pointing the services directly at files in
`/srv/.acme.sh`. This gives the services a stable path and lets acme.sh copy
new files into place on every renewal:

```bash
/srv/.acme.sh/acme.sh --home /srv/.acme.sh \
  --install-cert -d ttl.harstadmakers.no \
  --fullchain-file /srv/ttl-certs/fullchain.cer \
  --key-file /srv/ttl-certs/ttl.harstadmakers.no.key \
  --reloadcmd \
    "chown root:ttl-certs /srv/ttl-certs/fullchain.cer /srv/ttl-certs/ttl.harstadmakers.no.key && \
     chmod 0644 /srv/ttl-certs/fullchain.cer && \
     chmod 0640 /srv/ttl-certs/ttl.harstadmakers.no.key && \
     systemctl try-restart ttl-api.service ttl-frontend.service"
```

8. Utløpsdatoen må holdes oppdatert, da Discord botten vi varsle fra og med 30 dager før utløp. 

MERK! Sertifikatene fornyes langt oftere, for tiden hver 90. dag.

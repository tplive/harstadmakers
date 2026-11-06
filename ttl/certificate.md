# Utstede og fornye sertifikat for TTL

Vi bruker [Domeneshop](https://domene.shop) som DNS-leverandør og
[acme.sh](https://github.com/acmesh-official/acme.sh) for Let's Encrypt-
sertifikatet.

`acme.sh` installeres og kjøres av administratorbrukeren `my-admin-user`. Både
acme.sh-data og sertifikatene ligger under `/opt/acme.sh`:

```text
/opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer
/opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key
```

Backend og frontend kjører som `svc-ttl-api` og `svc-ttl-frontend`. De trenger
bare lesetilgang til de to sertifikatfilene.

## Installer acme.sh

Kjør installasjonen som `my-admin-user`, uten `sudo`:

```bash
git clone --depth 1 https://github.com/acmesh-official/acme.sh.git /tmp/acme.sh
cd /tmp/acme.sh
./acme.sh --install \
  --home /opt/acme.sh \
  --accountemail "my@example.com"
```

1. Logg inn på [Domeneshop](https://domene.shop) med din konto.
2. Følg instruksjonene: [Use Domeneshop DNS API to automatically issue a cert](https://github.com/acmesh-official/acme.sh/wiki/dnsapi2#85-use-domeneshop-dns-api-to-automatically-issue-a-cert)
3. "You'll have to get a Domeneshop API key and secret (<https://api.domeneshop.no/docs/>)."
4. "To generate credentials, visit this page after logging in to the control panel on our website: <https://www.domeneshop.no/admin?view=api>"
5. Noter token, secret og timestamp for utløp.
6. For manuelt å utstede sertifikat, kjør:
Kontroller installasjonen:

```bash
/opt/acme.sh/acme.sh --home /opt/acme.sh --version
/opt/acme.sh/acme.sh --home /opt/acme.sh --list
```

acme.sh oppretter normalt en cron-jobb for brukeren `my-admin-user`. Det er denne
jobben som senere fornyer sertifikatet.

## Utsted sertifikatet

Følg [Domeneshop DNS API-instruksjonene](https://github.com/acme.sh/acme.sh/wiki/dnsapi2#85-use-domeneshop-dns-api-to-automatically-issue-a-cert).
API-token og secret hentes fra
<https://www.domeneshop.no/admin?view=api>.

Ikke lagre credentials i repositoryet. Eksporter dem bare i shell-et som
brukeren `my-admin-user`:

```bash
export DOMENESHOP_Token=your_token_here
export DOMENESHOP_Secret=your_secret_here

/opt/acme.sh/acme.sh --home /opt/acme.sh \
  --issue --dns dns_domeneshop \
  -d ttl.harstadmakers.no \
  -d ttl-api.harstadmakers.no
```

## Gi tjenestene lesetilgang

Opprett en gruppe for sertifikatlesing og legg begge systembrukerne i gruppen:

```bash
sudo groupadd --system ttl-certs
sudo usermod --append --groups ttl-certs svc-ttl-api
sudo usermod --append --groups ttl-certs svc-ttl-frontend
sudo usermod --append --groups ttl-certs my-admin-user
```

Gi gruppen tilgang til domenekatalogen og sertifikatfilene:

```bash
sudo chgrp ttl-certs /opt/acme.sh/ttl.harstadmakers.no_ecc
sudo chmod 0750 /opt/acme.sh/ttl.harstadmakers.no_ecc

sudo chgrp ttl-certs \
  /opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer \
  /opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key
sudo chmod 0640 \
  /opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer \
  /opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key
```

`my-admin-user` må kunne traversere `/opt/acme.sh`, mens servicebrukerne skal ikke
ha generell tilgang til acme.sh-dataene. Bruk en ACL på toppkatalogen:

```bash
sudo chmod 0700 /opt/acme.sh
sudo setfacl -m u:my-admin-user:rwx,g:ttl-certs:x /opt/acme.sh
```

Installer `acl` først hvis `setfacl` ikke finnes:

```bash
sudo apt install acl
```

Kontroller at begge tjenestene kan lese sertifikatene:

```bash
sudo -u svc-ttl-api test -r /opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer
sudo -u svc-ttl-api test -r /opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key
sudo -u svc-ttl-frontend test -r /opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer
sudo -u svc-ttl-frontend test -r /opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key
```

## Installer fornyelseshook

Registrer de eksisterende sertifikatfilene som installasjonsmål. Ved hver
vellykkede fornyelse oppdaterer acme.sh filene, setter lesetilgang på nytt og
starter tjenestene:

```bash
/opt/acme.sh/acme.sh --home /opt/acme.sh \
  --install-cert -d ttl.harstadmakers.no \
  --fullchain-file /opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer \
  --key-file /opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key \
  --reloadcmd \
    "chgrp ttl-certs /opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer /opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key && \
     chmod 0640 /opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer /opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key && \
     systemctl restart ttl-api.service ttl-frontend.service"
```

`systemctl restart` i hooken må kjøres med tilstrekkelige rettigheter. Hvis
cron-jobben kjører som `my-admin-user`, gi kun denne kommandoen tilgang via sudo:

```bash
sudo tee /etc/sudoers.d/ttl-cert-renew >/dev/null <<'EOF'
my-admin-user ALL=(root) NOPASSWD: /usr/bin/systemctl restart ttl-api.service ttl-frontend.service
EOF
sudo chmod 0440 /etc/sudoers.d/ttl-cert-renew
sudo visudo --check
```

I så fall skal hookens siste kommando være:

```bash
sudo -n systemctl restart ttl-api.service ttl-frontend.service
```

Kontroller og test fornyelsesjobben:

```bash
/opt/acme.sh/acme.sh --home /opt/acme.sh --cron --dry-run
sudo journalctl -u ttl-api.service -u ttl-frontend.service -n 100 --no-pager
```

Konfigurasjonen og frontend-unit-filen skal bruke de samme filene:

```text
/opt/acme.sh/ttl.harstadmakers.no_ecc/fullchain.cer
/opt/acme.sh/ttl.harstadmakers.no_ecc/ttl.harstadmakers.no.key
```

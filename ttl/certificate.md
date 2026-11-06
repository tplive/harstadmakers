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
export DOMENESHOP_Token=your_token_here
export DOMENESHOP_Secret=your_secret_here
acme.sh --issue --dns dns_domeneshop -d ttl.example.com -d ttl-api.example.com
```

7. Vi beholder data lokalt på TTL serveren. I admin brukerens hjemmekatalog; lag `domeneshop_api.json`:

```json
{
    "domeneshop_token": "************",
    "domeneshop_secret": "*********************************************************",
    "domeneshop_expires": "2027-11-03 21:54:38"
}
```

8. Utløpsdatoen må holdes oppdatert, da Discord botten vi varsle fra og med 30 dager før utløp. 

MERK! Sertifikatene fornyes langt oftere, for tiden hver 90. dag.

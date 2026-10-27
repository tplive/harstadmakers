#import "@preview/merman:0.5.0": mermaid

#let codeblock(body) = block(
  fill: rgb("#f8fafc"),
  inset: 10pt,
  radius: 4pt,
  stroke: 0.5pt + rgb("#dfe7f1"),
  width: 100%,
)[
  #text(font: "DejaVu Sans Mono", size: 9pt, fill: rgb("#0f172a"))[#body]
]

#set page(
  paper: "a4",
  margin: (left: 18mm, right: 18mm, top: 18mm, bottom: 18mm),
  fill: rgb("#ffffff"),
)
#set text(
  font: "DejaVu Sans",
  size: 11pt,
  fill: rgb("#0f172a"),
  lang: "no",
)
#show link: underline

#set par(justify: true, leading: 0.75em)
#set heading(numbering: none)
#show heading.where(level: 1): it => block[
  #text(size: 18pt, weight: "medium", fill: rgb("#0f172a"))[#it.body]
  #v(0.55em)
]
#show heading.where(level: 2): it => block[
  #text(size: 15pt, weight: "medium", fill: rgb("#1e293b"))[#it.body]
  #v(0.4em)
]
#show heading.where(level: 3): it => block[
  #text(size: 13pt, weight: "medium", fill: rgb("#334155"))[#it.body]
  #v(0.3em)
]
#show heading.where(level: 4): it => block[
  #text(size: 11pt, weight: "medium", fill: rgb("#475569"))[#it.body]
  #v(0.25em)
]
#show heading.where(level: 5): it => block[
  #text(size: 10pt, weight: "medium", fill: rgb("#475569"))[#it.body]
  #v(0.2em)#import "@preview/merman:0.5.0": mermaid
]
#let codeblock(body) = block(
  fill: rgb("#f8fafc"),
  inset: 10pt,
  radius: 4pt,
  stroke: 0.5pt + rgb("#dfe7f1"),
  width: 100%,
)[
  #text(font: "DejaVu Sans Mono", size: 9pt, fill: rgb("#0f172a"))[#body]
]

#set page(
  paper: "a4",
  margin: (left: 18mm, right: 18mm, top: 18mm, bottom: 18mm),
  fill: rgb("#ffffff"),
)
#set text(
  font: "DejaVu Sans",
  size: 11pt,
  fill: rgb("#0f172a"),
  lang: "no",
)
#set par(justify: true, leading: 0.75em)
#set heading(numbering: none)
#show heading.where(level: 1): it => block[
  #text(size: 18pt, weight: "medium", fill: rgb("#0f172a"))[#it.body]
  #v(0.55em)
]
#show heading.where(level: 2): it => block[
  #text(size: 15pt, weight: "medium", fill: rgb("#1e293b"))[#it.body]
  #v(0.4em)
]
#show heading.where(level: 3): it => block[
  #text(size: 13pt, weight: "medium", fill: rgb("#334155"))[#it.body]
  #v(0.3em)
]
#show heading.where(level: 4): it => block[
  #text(size: 11pt, weight: "medium", fill: rgb("#475569"))[#it.body]
  #v(0.25em)
]
#show heading.where(level: 5): it => block[
  #text(size: 10pt, weight: "medium", fill: rgb("#475569"))[#it.body]
  #v(0.2em)
]
#show heading.where(level: 6): it => block[
  #text(size: 9pt, weight: "medium", fill: rgb("#475569"))[#it.body]
  #v(0.15em)
]
#show link: it => text(fill: rgb("#0f172a"), weight: "regular")[#it.body]
#align(center)[
  #text(size: 24pt, weight: "medium")[Harstad Makers TTL
  ]

  #text(size: 14pt, weight: "regular")[
    (Tilgang Til Lokalene)]
]
#v(1.2em)
#line(length: 100%, stroke: 0.8pt + rgb("#dfe7f1"))
#v(1.2em)

#outline(title: [Innholdsfortegnelse])

#pagebreak()
= Introduksjon
Dette er en webapplikasjon som på Fiffig Vis#emoji.tm lar brukerne låse opp døren til lokalet vårt. Løsningen er meget sikker, benytter public key kryptering og er ikke eksponert til Internett. Den som vil bryte seg inn hos oss må enten være en #emph[eksepsjonelt korttenkt, men begavet hacker]... eller en #emph[smarting med et brekkjern for hånden]... Get it? #emoji.face

Lokal tilgangskontroll for å komme inn i våre lokaler. Tilgangskontrollen vil etter hvert bli koblet opp til medlemsregisteret, og kun medlemmer vil kunne få tilgang.

= Hvordan virker det?

Designet er superenkelt for brukeren, og avansert nok til å være sikkert:

#enum[
  Medlemmer må registrere seg før de kan bruke tilgangskontrollen. Dette gjøres èn gang, men må gjentas hvis man bytter telefon eller rensker nettleseren på telefonen, da tilgangskoden lagres der. En bruker kan ha flere nettlesere godkjent. Feks en mobil og en på sin pc. #strong[Det er ingen innlogging eller passord å huske.]
][
  Registreringen gjør at medlemmets tilgangsnøkkel lagres på server.
][
  Når man vil låse opp døren, åpner man nettstedet fra mobilen sin mens man står på utsiden.
][
  Nettleser og server utveksler litt data, og nettleseren må bevise at den har rett tilgangsnøkkel ved å dekryptere en utfordring fra server.
][
  Når nettleser svarer med korrekt dekryptert utfordring, låses døra opp! #emoji.face
]

Teknologien som brukes kalles Public Key Encryption. Dersom du er nysgjerrig kan du f.eks. lese mer om dette her: https://www.geeksforgeeks.org/computer-networks/public-key-encryption/

= Hvilke hardware-komponenter inngår?
#show link: underline
#list(
  [En fysisk dørlås (eng. "electric door strike") på ytterdøra som kan aktiveres med å gi den 12V spenning. #link("https://fyndiq.no/produkt/dc12v-elektrisk-las-for-port-elektrisk-dorapner-for-alle-typer-dorer-rustfritt-stal-98da778d3140538a/")[Feks en sånn]],
  [Et relè som lukker kretsen og gir dørlåsen spenning. #link("https://fyndiq.no/produkt/rele-srd-5vdc-sl-c-1-vekselkontakt-5v-dc-spole-10a-240vac-for-elektronikkprosjekter-5cf8b376f2994921/")[Feks et sånt]],
  [En server som er: (#link("https://cdon.no/produkt/raspberry-pi-3-model-b-enkortsdatamaskin-12-ghz-1-gb-ram-9a53d04f738d4a2a/")[Eksempel, vi har sånne fra før])
    #list(
      [fysisk plassert inne i lokalene],
      [fysisk koblet til dørlåsen via relèet],
      [fysisk koblet til relèet via en GPIO-pinne],
      [når korrekt opplåsing skjer, lukker den relèet ved å gi det 5V spenning, som aktiverer dørlåsen.],
    )],
  [En gammel (pent brukt) PC strømforsyning som gir 5V til serveren og relèet, og 12V til dørlåsen. (Det har vi sikkert fra før)],
  [En UPS (Uninterruptable Power Supply) som holder server, wifi og dørlås operative selv om strømmen skulle gå. (Har kjøpt en brukt)],
  [Internett-ruteren, som muliggjør nedlasting av sertifikater for serveren. Mer om det lenger ned. (den har vi fra før)],
  [Den opprinnelige Yale dørlåsen på døra, som "master lock":
    #list(
      [#strong[fortsatt fullt fungerende]],
      [#strong[helt uavhengig av denne løsningen]],
      [#strong[fortsatt styrt av huseier]],
    )],
)

= Hvilken software inngår?

#list(
  [Serveren leverer en nettside og et API for dørkontrollen.],
  [Serveren henter sertifikat for #raw("ttl.harstadmakers.no") og #raw("ttl-api.harstadmakers.no") fra Let's Encrypt via Certbot.],
  [Serveren kjører Nginx som TLS terminator. Se lenger ned i dokumentet.],
  [Serveren er konfigurert med IP-adresse på Harstad Makers sitt wifi-nettverk og er kun tilgjengelig når man er tilknyttet dette.],
  [Medlemmets PC eller mobiltelefon lagrer nøkler i nettleserens #raw("localStorage") som enkelt kan inspiseres (se lenger ned).],
)

= Teknisk Software-arkitektur

#list(
  [Frontend: statisk HTML, CSS og JavaScript som serveres lokalt av serveren via en enkel webserver i lokalnettverket, over TLS med korrekt sertifikat.],
  [Backend: Rust + Axum-tjeneste for lagring av medlemmer, generering av utfordringer og autorisasjonskontroll.],
  [Persistens: JSON-baserte logger og medlemsdata i backend-data-mappen. #strong[Kommer: En database som erstatter filene!]],
)

== Aktører

#list(
  [Serveren],
  [Nettleseren],
  [Brukeren],
  [Tilgangsansvarlig],
)

== Registrering

@figur1 viser flyten:

#enum[
  Brukeren kobler til wifi og åpner nettsiden.
][
  Brukeren taster inn navn og epost og trykker på "Registrering". Dette genererer en public/private nøkkelpar og en UUID.
][
  Privat nøkkel og UUID lagres i nettleserens local storage.
][
  Navn, epost, offentlig nøkkel og UUID sendes til serveren.
][
  Server svarer med mottaksmelding og sin public key som også lagres i nettleserens local storage.
]

#figure(
  mermaid(
    "
  sequenceDiagram
     participant Bruker
     participant Nettleser
     participant Server
     participant Medlemsdata

     Bruker->>Nettleser: Åpner nettsiden og fyller ut navn og epost
     Bruker->>Nettleser: Trykker «Registrering»
     Nettleser->>Nettleser: Genererer nøkkelpar og UUID
     Nettleser->>Nettleser: Lagrer privat nøkkel og UUID i local storage
     Nettleser->>Server: Sender navn, epost, offentlig nøkkel og UUID
     Server->>Medlemsdata: Lagrer registreringen
     Medlemsdata-->>Server: Bekrefter lagring
     Server-->>Nettleser: Sender mottaksmelding og serverens offentlige nøkkel
     Nettleser->>Nettleser: Lagrer serverens offentlige nøkkel i local storage
     Nettleser-->>Bruker: Viser at registreringen er mottatt
",
  ),
  caption: [Registreringsflyt],
)<figur1>

== Aktivering

#enum[
  Brukeren har fullført registrering.
][
  Tilgangsansvarlig logger seg på server og finner brukerens registrering ved epost/navn.
][
  Tilgangsansvarlig verifiserer med å kontrollere siste siffer i brukerens registrerte UUID.
][
  Tilgangsansvarlig setter "active": "true" på registreringen. #strong[Kommer: et GUI for å administrere registreringene]
]

#mermaid(
  "
  sequenceDiagram
     participant Bruker
     participant Tilgangsansvarlig
     participant Server
     participant Medlemsdata

     Bruker->>Tilgangsansvarlig: Fullfører registreringen
     Tilgangsansvarlig->>Server: Logger inn og søker etter registreringen
     Server->>Medlemsdata: Henter registreringer
     Medlemsdata-->>Server: Returnerer registreringer
     Server-->>Tilgangsansvarlig: Viser registrering funnet med epost eller navn
     Tilgangsansvarlig->>Bruker: Ber om siste siffer i UUID
     Bruker-->>Tilgangsansvarlig: Oppgir siste siffer i UUID
     Tilgangsansvarlig->>Tilgangsansvarlig: Verifiserer UUID-sifferet
     Tilgangsansvarlig->>Server: Setter «active»: «true»
     Server->>Medlemsdata: Lagrer aktiv registrering
     Medlemsdata-->>Server: Bekrefter oppdateringen
     Server-->>Tilgangsansvarlig: Bekrefter at brukeren er aktivert
",
)

== Opplåsing av døra

#list(
  [Tilgangsflyt:],
)

#enum[
  Brukeren kobler til trådløsnettet og åpner nettsiden.
][
  Nettleseren sjekker local storage og finner at den er registrert.
][
  Nettleseren henter brukerens private nøkkel og UUID fra local storage.
][
  Nettleseren genererer en client-nonce som den krypterer med serverens public key og sender denne sammen med UUID til serveren og ber om en _"registration-challenge"_.
][
  Server sjekker om tilsendt UUID finnes i databasen.
][
  Dersom den ikke finnes#footnote[Dersom den ikke finnes har den kanskje aldri eksistert der, eller rydderutinen har slettet den. Uansett må bruker registrere seg på nytt.],
  - serveren dekrypterer client-nonce og returnerer den som bevis på at den er riktig server.
  - serveren gir beskjed til nettleseren om at brukeren må registrere seg.
  - nettleseren viser brukeren en knapp for "Slett registrering", som vil fjerne alle relevante verdier fra local storage i nettleseren og nullstille nettleseren.
][
  Dersom den finnes,
  - serveren dekrypterer client-nonce og returnerer den som bevis på at den er riktig server.
  - genererer serveren en server-nonce som den krypterer med brukerens registrerte public key og
  - sender kryptert server-nonce til nettleser som challenge.
][
  Nettleseren dekrypterer server-nonce med brukerens private nøkkel og sender dekryptert verdi tilbake til server som bevis på at den er den riktige nettleseren.
][
  Serveren validerer dekryptert server-nonce og gir melding tilbake til nettleseren om at brukeren er klarert for å låse opp døra.
][
  Brukeren ser nå en "Lås opp døra" knapp i nettleseren.
][
  Når bruker trykker på "Lås opp døra", gjentas stegene over for ny validering, men denne gang som en _"unlock-challenge"_.
][
  Server aktiverer relèet som åpner dørlåsen i 5 sekunder (konfigurerbart).
]

#mermaid(
  "
  sequenceDiagram
     participant Bruker
     participant Nettleser
     participant Server
     participant Medlemsdata
     participant Relè
     participant Dørlås

     Bruker->>Nettleser: Kobler til trådløsnettet og åpner nettsiden
     Nettleser->>Nettleser: Henter privat nøkkel og UUID fra local storage
     Nettleser->>Nettleser: Genererer og krypterer client-nonce
     Nettleser->>Server: Sender UUID og kryptert client-nonce
     Server->>Medlemsdata: Sjekker om UUID finnes
     Medlemsdata-->>Server: Returnerer registreringen

     alt UUID finnes ikke
        Server->>Server: Dekrypterer client-nonce
        Server-->>Nettleser: Ber brukeren registrere seg på nytt
        Nettleser-->>Bruker: Viser «Slett registrering»
     else UUID finnes
        Server->>Server: Dekrypterer client-nonce
        Server->>Server: Genererer server-nonce
        Server->>Server: Krypterer server-nonce med brukerens offentlige nøkkel
        Server-->>Nettleser: Sender kryptert server-nonce som challenge
        Nettleser->>Nettleser: Dekrypterer server-nonce med privat nøkkel
        Nettleser->>Server: Sender dekryptert server-nonce
        Server->>Server: Validerer server-nonce
        Server-->>Nettleser: Bekrefter at brukeren er klarert
        Nettleser-->>Bruker: Viser «Lås opp døra»
        Bruker->>Nettleser: Trykker «Lås opp døra»
        Nettleser->>Server: Gjentar valideringen med unlock-challenge
        Server->>Relè: Aktiverer relèet i 5 sekunder
        Relè->>Dørlås: Gir dørlåsen spenning
        Dørlås-->>Bruker: Døren låses opp
     end
",
)

#super[1]: Dersom den ikke finnes har den kanskje aldri eksistert der, eller rydderutinen har slettet den. Uansett må bruker registrere seg på nytt.

= Sikkerhetsmodell

Noen nøkkelpunkter som gjør løsningen svært sikker og robust:

#list(
  [Nettleser genererer og lagrer sin egen private nøkkel, som aldri forlater nettleseren.],
  [Server genererer og lagrer sin egen private nøkkel, som aldri forlater serveren.],
  [Nettleser representerer sin identitet med å sende over sin offentlige nøkkel til serveren ved registrering.],
  [Server representerer sin identitet med å sende over sin offentlige nøkkel til nettleseren ved registrering.],
  [Nettleser lagrer ikke selv sin offentlige nøkkel, men kan bevise sin identitet ved å dekryptere nonce sendt fra server.],
  [Server kan bevise sin identitet ved å dekryptere nonce sendt fra nettleser.],
  [Trafikk mellom nettleser på mobil og server er kryptert over HTTPS.],
  [Løsningen er ikke eksponert på Internett, brukeren må være tilknyttet lokalnettverket.],
  [Etter registrering må en administrator flagge registreringen som aktiv før brukeren kan låse opp døren.],
)

== Nettleser

Server frontend og backend har begge TLS-sertifikater utstedt fra Let's Encrypt. Disse sikrer at nettleser validerer serveren korrekt, og vil gi sertifikat-feilmelding dersom en man-in-the-middle forsøker å lure brukeren.

Bruker fyller kun ut (for)navn og epost i nettleser. Nettleseren genererer en UUID og et nøkkelpar (privat og offentlig nøkkel). #strong[Kun UUID og privat nøkkel] lagres i nettleseren. #strong[Offentlig nøkkel sendes til server] som lagrer denne sammen med navn, epost, tidsstempel.

Man kan enkelt inspisere data som er lagret i nettleser, via F12 Developer Tools. Se under Storage, navn på server (enten localhost:8088, eller det faktiske navnet, ettersom.)

#figure(
  image("localstorage.png", width: 65%),
  caption: [Data lagret i nettleserens local storage],
)

I teorien kan en angriper (MiTM) sende over javascript som forbigår server-sjekken, og lurer brukeren til å oppgi sitt navn og epost-adresse. Og sikkert lure fra dem kontonummer og fødselsnummer og hva det skal være. Sannsynligheten er lav. Og dette vil ikke la angriper åpne døren.

== API

Ved forespørsel fra nettleser genererer backend en nonce og krypterer den med medlemmets offentlige nøkkel. Den sender kryptert nonce til klienten. Nettleseren dekrypterer nonce med den private nøkkelen og sender klarteksten tilbake for verifisering. Når klarteksten matcher lagret nonce, godkjenner backend døråpning. Brukeren ber nå om å få åpnet døren, og dette trigger dørlåsen.

Backend har en forhåndsgenerert Ed25519 SSH-nøkkel i #raw("~/.ssh/id_ed25519") og en offentlig nøkkel i #raw("~/.ssh/id_ed25519.pub"). Den offentlige

- Certbot obtains and renews the TLS certificate for the RPI hostname.
- The mobile browser trusts the RPI certificate through its built-in trusted Certificate Authorities.
- Browser-to-RPI communication uses HTTPS, providing encrypted transport and server authentication.

= Let's Encrypt certbot for frontend og backend

Let's Encrypt verifiserer at man kontrollerer domenet, og utsteder sertifikater som verifiseres av nettleser.
Sertifikatet inneholder ikke IP-adresse, så man kan (via DHCP) publisere lokale DNS-oppslag som gir lokale IP-adresser for serveren.

I følge dokumentasjonen til [Let's Encrypt](https://letsencrypt.org/docs/challenge-types/) er det ulike typer challenge som kan benyttes for å automatisere sertifikat-utstedelse.

Registrar og DNS leverandør må støtte "DNS-01 challenge" for at sertifikatene skal kunne automatiseres. Vi har verifisert at vår leverandør gjør det. Da konfigurerer man en certbot (feks. [acme.sh](https://github.com/acmesh-official/acme.sh/) med token og secret for DNS-leverandøren, og så publiserer serveren en DNS TXT record som Let's Encrypt bruker som bevis på at man kontrollerer domenet.

Alternativ: HTTP-01 challenge - dette krever at port 80 er åpent inn til serveren, og publiserer en tekstfil ut på Internett. Brukes dersom man ikke kommer i mål med DNS-01.

== Hvordan sørge for at TLS-sertifikatene kan utstedes automatisk

Vi bruker to DNS-navn:

#list(
  [#raw("ttl.harstadmakers.no") → frontend],
  [#raw("ttl-api.harstadmakers.no") → backend],
)

Klientene på vårt wifi-nettverk har fått konfigurasjon med lokal DNS server som holder en stub-sone for harstadmakers.no, og videreformidler alle andre oppslag til harstadmakers.no som den ikke kjenner til.

I denne DNS-serveren peker vi disse navnene til de lokale IP-adressene til server, for eksempel:

#codeblock[
  ttl.harstadmakers.no      -> 192.168.1.50
  ttl-api.harstadmakers.no  -> 192.168.1.50
]

Dette gjør at medlemmer på det lokale nettverket går til:

#codeblock[
  https://ttl.harstadmakers.no
  https://ttl-api.harstadmakers.no
]

som om de var tilgjengelige på Internett.

== Nginx som TLS-terminator (ved alternativ løsning HTTP-01 challenge)

På server kjører vi Nginx og konfigurerer den som reverse proxy til frontend og backend.

Det eneste som skal være åpent mot Internett er port 80 for Let's Encrypt HTTP-01 challenge. Alt annet skal blokkeres av ruteren/firewallen og være utilgjengelig fra Internett.

Eksempel:

#codeblock[
  \# Port 80 er kun åpent for ACME challenge.
  \# Ingen andre tjenester skal være publisert direkte mot Internett.
  server {
  listen 80;
  server_name ttl.harstadmakers.no;

  location /.well-known/acme-challenge/ {
  root /var/www/letsencrypt;
  }

  return 301 https://\$host\$request_uri;
  }

  server {
  listen 443 ssl;
  server_name ttl.harstadmakers.no;

  ssl_certificate /etc/letsencrypt/live/ttl.harstadmakers.no/fullchain.pem;
  ssl_certificate_key /etc/letsencrypt/live/ttl.harstadmakers.no/privkey.pem;

  location / {
  proxy_pass http://127.0.0.1:8088;
  proxy_set_header Host \$host;
  proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
  proxy_set_header X-Forwarded-Proto https;
  }
  }

  server {
  listen 80;
  server_name ttl-api.harstadmakers.no;

  location /.well-known/acme-challenge/ {
  root /var/www/letsencrypt;
  }

  return 301 https://\$host\$request_uri;
  }

  server {
  listen 443 ssl;
  server_name ttl-api.harstadmakers.no;

  ssl_certificate /etc/letsencrypt/live/ttl-api.harstadmakers.no/fullchain.pem;
  ssl_certificate_key /etc/letsencrypt/live/ttl-api.harstadmakers.no/privkey.pem;

  location / {
  proxy_pass http://127.0.0.1:3000;
  proxy_set_header Host \$host;
  proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
  proxy_set_header X-Forwarded-Proto https;
  }
  }
]

== ACME / certbot

Installer Certbot:

#codeblock[
  sudo apt update
  sudo apt install -y certbot python3-certbot-nginx
]

Hent sertifikatene:

#codeblock[
  sudo certbot --nginx -d ttl.harstadmakers.no -d ttl-api.harstadmakers.no
]

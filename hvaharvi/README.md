# Makerspace-inventarsystem med lokal AI

## Mål

Bygge et system som gjør det ekstremt enkelt for medlemmer av et makerspace å:

- Registrere hvilke gjenstander som finnes
- Registrere hvor gjenstandene befinner seg
- Finne frem til gjenstander senere
- Bruke naturlig språk på norsk for å søke i inventaret

Systemet bør være selvhostet, fungere lokalt og ikke være avhengig av skytjenester.

---

# Ønsket brukeropplevelse

## Registrering av en gjenstand

Ideell arbeidsflyt:

1. Brukeren skanner en QR-kode som representerer en lokasjon.
2. Brukeren tar ett eller flere bilder av gjenstanden.
3. En lokal AI analyserer bildet.
4. AI foreslår:
   - Gjenstandstype
   - Kategori
   - Relevante emneord (tags)
   - Beskrivelse
5. Brukeren bekrefter eller korrigerer forslagene.
6. Gjenstanden lagres i databasen.

Målet er at registrering skal ta under 10 sekunder.

---

## Finne en gjenstand

Brukeren skal kunne bruke:

### Tradisjonelt søk

Eksempler:

- Drill
- Arduino
- Symaskin
- 10 mm trebor

### Naturlig språk

Eksempler:

> Har vi et 10 mm bor for treverk?

> Hvor ligger symaskinen?

> Har vi noen Arduino Nano?

> Hvor finner jeg keramiske motstander på 220 ohm?

AI-assistenten skal forstå spørsmålet, søke i inventarbasen og svare på norsk.

---

# Lokasjonsstruktur

Alle lagringssteder organiseres i et hierarki.

Eksempel:

```text
Verksted
├── Elektronikkrom
│   ├── Komponenthyller
│   │   ├── Skuffeboks 1
│   │   │   ├── A01
│   │   │   ├── A02
│   │   │   └── A03
│
└── Maskinrom
    ├── CNC-hylle
    ├── 3D-printerreol
    └── Verktøyskap
```

Mer detaljerte nivåer kan legges til:

```text
Verktøyskap
 └── Skuff B
      └── Rom 3
```

Alle objekter må kunne knyttes til én bestemt lokasjon.

---

# QR-koder

Det anbefales sterkt å merke alle oppbevaringssteder med QR-koder.

Eksempler:

- Hylle A
- Hylle B
- Verktøyskap
- Skuff A01
- Skuff A02
- Elektronikkbenk
- Materiallager

Arbeidsflyt:

1. Skann QR-kode.
2. Ta bilde.
3. Godkjenn AI-forslag.
4. Lagre.

Fordeler:

- Rask registrering
- Færre feil
- Ensartet lokasjonsstruktur
- Ingen behov for manuell innskriving av lagringssted

---

# Datamodell for inventar

Eksempel:

```json
{
  "name": "10 mm trebor",
  "location": "Maskinrom/Verktøyskap/Skuff B3",
  "photos": [
    "photo1.jpg"
  ],
  "tags": [
    "bor",
    "trebearbeiding",
    "10mm"
  ]
}
```

Forslag til felter:

| Felt | Beskrivelse |
|--------|------------|
| Navn | Beskrivende navn |
| Kategori | Verktøy, komponent, maskin osv. |
| Lokasjon | Hvor gjenstanden befinner seg |
| Bilder | Ett eller flere bilder |
| Beskrivelse | Fritekst |
| Antall | Antall enheter |
| Eier | Valgfritt |
| Opprettet dato | Når registreringen ble laget |
| Sist verifisert | Når plasseringen sist ble bekreftet |
| AI-tagger | Automatisk genererte emneord |

---

# Bildegjenkjenning (Computer Vision)

AI skal hjelpe brukerne, ikke automatisk bestemme alt.

Brukeren skal alltid kunne godkjenne eller korrigere forslag.

---

## Florence-2

Sterk modell for:

- Bildebeskrivelse
- Objektidentifikasjon
- Visuell forståelse

Mulige bruksområder:

- Foreslå navn på gjenstander
- Lage beskrivelser
- Identifisere verktøy og utstyr

Eksempel:

Bilde av en drill kan gi:

```text
Elektrisk drill
Bor
Verktøykasse
```

---

## YOLOv11

Egnet for:

- Rask objektdeteksjon
- Lokale installasjoner
- Moderat maskinvare

Kan identifisere:

- Skrutrekkere
- Bor
- Multimetre
- Tenger
- Loddebolter
- Arduino-kort

---

## Grounding DINO

God til å finne objekter uten spesialtrening.

Eksempler:

```text
Finn motstand
Finn oscilloskop
Finn Arduino
Finn symaskin
```

---

# AI skal bruke sannsynlighet

I stedet for å registrere automatisk:

```text
Oppdaget:

✅ Bor (95 %)
✅ Diameter 10 mm (82 %)
⚠ Trebor (61 %)
```

Brukeren bekrefter før lagring.

Dette hindrer at databasen fylles med feilaktige klassifiseringer.

---

# Søkelaget

Inventardatabasen er systemets fasit.

LLM-en skal ikke lagre informasjon selv.

Arbeidsflyt:

```text
Bruker
  ↓
LLM
  ↓
Inventardatabase
  ↓
Svar på norsk
```

Eksempel:

Brukeren spør:

> Har vi et 10 mm trebor?

LLM genererer en databasespørring:

```sql
SELECT *
FROM inventory
WHERE
diameter = 10
AND category = 'Drill Bit'
AND material = 'Wood';
```

Resultat:

```text
Funnet 2 treff.

Verktøyskap / Skuff B3
Verktøyvegg / Krok 12
```

LLM svarer:

> Ja. Vi har to registrerte 10 mm trebor. Ett ligger i verktøyskapet i skuff B3, og ett henger på verktøyveggen på krok 12.

---

# Semantisk søk

Vanlig tekstsøk er ikke nok.

Eksempel:

Registrert:

```text
10 mm trebor
```

Brukeren søker:

```text
drillbit til treverk
```

Systemet bør forstå at dette er det samme.

---

## Embedding-modeller

Aktuelle modeller:

- BGE-M3
- Nomic Embed
- Arctic Embed

Disse lager vektorrepresentasjoner av tekst.

---

## Vektordatabaser

### PostgreSQL + pgvector

Anbefalt standardvalg.

Fordeler:

- Enkelt å drifte
- Ingen egen søkeplattform nødvendig
- Strukturert og semantisk søk i samme database

Alternativer:

- Qdrant
- Weaviate

---

# Norsk språkstøtte

Lokale språkmodeller som fungerer godt på norsk:

## Qwen 3

Anbefalt hovedmodell.

Fordeler:

- God generell resonnering
- God forståelse av norsk
- Kan kjøres lokalt

---

## Llama 3.3

God flerspråklig støtte.

---

## Mistral Small

Effektiv modell med moderate maskinvarekrav.

---

## Gemma 3

Lettvektsmodell som fungerer godt på mindre maskiner.

---

# Eksisterende systemer som kan gjenbrukes

## Snipe-IT

Open source-system for ressursforvaltning.

Har allerede:

- Lokasjoner
- Brukerstyring
- Inventarhåndtering

Kan utvides med AI-funksjoner.

---

## Homebox

Open source-system for inventar.

Har allerede:

- Bilder
- Kategorier
- Lokasjoner

Ligner svært mye på behovet i et makerspace.

---

## Grocy

Lagersystem opprinnelig laget for husholdninger.

Kan fungere, men passer mindre godt for utstyr og verktøy.

---

# Foreslått teknologistack

## Frontend

Progressive Web App (PWA)

Krav:

- Mobilvennlig
- Kameraintegrasjon
- QR-skanning

---

## Backend

FastAPI

Fordeler:

- Moderne Python-rammeverk
- Enkel AI-integrasjon
- God ytelse

---

## Database

PostgreSQL

Utvidelser:

- pgvector

---

## AI-modeller

### Bildegjenkjenning

- Florence-2
- YOLOv11
- Grounding DINO

### Språkmodell

- Qwen 3 14B

---

## Fillagring

MinIO

Brukes til:

- Bilder
- Vedlegg
- Backup

---

## Søk

PostgreSQL + pgvector

Støtter:

- Tradisjonelt søk
- Semantisk søk
- AI-assistert søk

---

# Infrastruktur

## Minimum

- Moderne flerkjernet CPU
- 32 GB RAM
- 1 TB SSD

---

## Anbefalt

- NVIDIA RTX 4060 eller bedre
- 64 GB RAM
- 2 TB SSD

Kan kjøres på:

- Gjenbrukt arbeidsstasjon
- Lokal server
- Mini-PC med dedikert GPU

---

# Viktigste designprinsipp

AI er ikke den vanskeligste delen.

Den største utfordringen er å få medlemmene til faktisk å registrere ting.

Suksesskriterier:

- Registrering tar mindre enn 10 sekunder.
- QR-kode brukes før bilde tas.
- AI minimerer manuell inntasting.
- Lokasjonshierarkiet er enkelt og konsekvent.
- Systemet fungerer godt på mobil.

Dersom registrering er rask og friksjonsfri, vil resten av systemet være relativt enkelt å få til.

---

# Mulige fremtidige utvidelser

## Strekkodeskanning

Automatisk identifisering av:

- Verktøy
- Komponenter
- Materialer

---

## OCR

Lesing av:

- Verdier på motstander
- IC-nummer
- Produktetiketter
- Serienumre

---

## Utlånssystem

Registrere:

- Hvem som lånte noe
- Når det ble lånt
- Forfallsdato

---

## "Sist sett"-sporing

Mulighet for å se:

- Når en gjenstand sist ble registrert
- Hvem som flyttet den

---

## Duplikatdeteksjon

Systemet oppdager:

- Like bilder
- Like objekter
- Potensielle duplikatregistreringer

---

## Integrasjon mot komponentdatabaser

Mulige kilder:

- DigiKey
- Mouser
- Farnell
- LCSC

Eksempel:

Systemet leser:

```text
LM2596
```

og henter automatisk:

- Datablad
- Beskrivelse
- Kategori
- Teknisk informasjon

---

# Langsiktig visjon

Et medlem skal kunne gå inn i verkstedet, ta et bilde av en gjenstand og få den registrert på under ti sekunder.

Et år senere skal et annet medlem kunne spørre:

> Har vi en keramisk motstand på 220 ohm?

og få svaret:

> Ja. Vi har fem slike motstander. De ligger i Elektronikkrom → Komponenthyller → Skuffeboks 3 → Skuff B12.

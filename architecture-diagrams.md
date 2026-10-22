# Harstad Makers architecture diagrams

## 1) High-level system overview

```mermaid
flowchart LR
    subgraph LocalNetwork[Local network]
        Mobile[Member mobile browser]
        PiSite[RPI-hosted web site]
        API[RPI Rust API]
    end

    RPI[RPI server identity<br/>Ed25519 / PKI signing key]
    MemberKey[Member key pair<br/>private key stays in browser]
    Door[RPI door controller]
    Lock[Door lock / relay / actuator]

    Mobile -->|load web UI| PiSite
    PiSite -->|API requests| API
    API -->|sign browser challenge| Mobile
    Mobile -->|verify RPI signature| RPI
    API -->|encrypt nonce with member public key| Mobile
    Mobile -->|decrypt with member private key| MemberKey
    Mobile -->|return decrypted nonce| API
    API -->|verified unlock command| Door
    Door --> Lock
```

## 2) Local deployment

```mermaid
flowchart TB
    subgraph RPI[RPI]
        Certbot[Certbot]
        Certificate[TLS certificate<br/>issued for RPI hostname]
        WebServer[Local HTTPS web server]
        RustAPI[Local Rust API]
        MemberStore[members.json<br/>member public keys and status]
        ServerIdentity[Ed25519 / PKI signing key]
        DoorController[GPIO / relay controller]
        DoorHardware[Door relay / lock]

        Certbot -->|obtains and renews| Certificate
        Certificate --> WebServer
        WebServer --> RustAPI
        RustAPI --> MemberStore
        RustAPI --> ServerIdentity
        RustAPI --> DoorController
        DoorController --> DoorHardware
    end

    MobileBrowser[Member mobile browser]
    BrowserCAs[Browser trusted Certificate Authorities]

    BrowserCAs -->|trusts certificate issuer| MobileBrowser
    MobileBrowser <-->|HTTPS<br/>encrypted and authenticated| WebServer
```

- The website and API run locally on the RPI.
- Certbot obtains and renews the TLS certificate for the RPI hostname.
- The mobile browser trusts the RPI certificate through its built-in trusted Certificate Authorities.
- Browser-to-RPI communication uses HTTPS, providing encrypted transport and server authentication.

## 3) Member registration and unlock flow

```mermaid
sequenceDiagram
    participant User as Member mobile browser
    participant RPI as RPI
    participant Data as Member data store
    participant Relay as 12V relay
    participant Lock as Electronic door strike

    User->>User: Generate member key pair
    User->>RPI: POST /api/members { public_key, browser_challenge }
    RPI->>RPI: Sign browser challenge with Ed25519 private key
    RPI-->>User: Registration response + RPI public key + signature
    User->>User: Accept and store RPI public key and signature
    RPI->>Data: Store member public key and registration

    User->>RPI: POST /api/challenge { member_id, browser_challenge }
    RPI->>RPI: Sign browser challenge with Ed25519 private key
    RPI->>RPI: Generate nonce and encrypt it with member public key
    RPI-->>User: challenge_id + encrypted_nonce + signature
    User->>User: Verify RPI signature
    User->>User: Decrypt nonce with member private key
    User->>RPI: POST /api/door/unlock { member_id, challenge_id, decrypted_nonce }
    RPI->>RPI: Verify nonce, member status, and challenge
    RPI->>Relay: Trigger 12V relay
    Relay->>Lock: Switch power to door strike
    RPI-->>User: approved: true
```

## 4) Internal component responsibilities

```mermaid
flowchart LR
    subgraph RPIHost[RPI]
        WebUI[Web interface]
        RustAPI[Rust API]
        MemberData[Member records and public keys]
        ServerKey[Ed25519 server signing key]
        Controller[GPIO / relay control]
    end

    subgraph Registration[Registration]
        BrowserKey[Generate browser key pair]
        RegistrationChallenge[Browser-generated challenge]
        RPITrust[Accept and store RPI public key<br/>and registration signature]
    end

    subgraph AccessFlow[Door access]
        AccessChallenge[Browser-generated challenge]
        Nonce[Generate and encrypt one-time nonce]
        ServerSignature[Sign challenge response]
        Verify[Verify stored RPI signature and decrypted nonce]
        Allow[Allow unlock]
    end

    WebUI --> RustAPI
    BrowserKey --> RegistrationChallenge
    RegistrationChallenge --> RustAPI
    RustAPI --> ServerKey
    ServerKey --> RPITrust
    RustAPI --> MemberData
    AccessChallenge --> RustAPI
    RustAPI --> Nonce
    RustAPI --> ServerSignature
    RPITrust --> Verify
    Nonce --> Verify
    ServerSignature --> Verify
    Verify --> Allow
    Allow --> Controller
    Controller --> Door[Physical lock]
```

## 5) Recommended architecture

```mermaid
flowchart TB
    subgraph LocalNetwork[Local network]
        Mobile[Member mobile browser]
        RPIWebSite[RPI web site]
        API[RPI Rust API]
    end

    ServerIdentity[RPI PKI identity<br/>Ed25519 signing key]
    MemberData[Member public keys and registration data]
    DoorController[RPI door relay controller]
    Lock[Door hardware]

    Mobile --> RPIWebSite
    Mobile -->|signed registration and access requests| API
    API -->|sign challenge responses| ServerIdentity
    API --> MemberData
    API -->|validated unlock request| DoorController
    DoorController --> Lock
```

## 6) Security boundary

```mermaid
flowchart LR
    Browser[Member browser]
    RPIWebSite[RPI web site]
    RustAPI[RPI Rust API]
    ServerKey[RPI Ed25519 / PKI identity]
    MemberKeys[Member public keys in members.json]
    Door[Door hardware]

    Browser --> RPIWebSite
    Browser -->|registration and access requests| RustAPI
    RustAPI -->|sign challenge responses| ServerKey
    RustAPI --> MemberKeys
    RustAPI -->|unlock only after verification| Door

    classDef secure fill:#d9f7be,stroke:#389e0d,stroke-width:2px;
    class RPIWebSite,RustAPI,ServerKey,MemberKeys,Door secure;
```

## Notes
- The server is also called "RPI" in our documentation.
- The member website and API run on the server.
- On registration:
  - the client browser generates a public/private key pair.
  - the client stores its own private key and sends its public key to the server. The public key is not stored in the browser.
  - the server receives the public key and generates a member UUID and sends it along with its public key to the client.
  - the client stores three pieces of information; its private key, the servers public key and the member UUID.
- The door unlock server is only accessible on our local LAN/Wifi.
- The server authenticates its responses by signing browser-generated challenges with its Ed25519/PKI private key.
- The browser verifies the server signature using the trusted server public key before accepting registration or access responses.
- The member key pair proves possession of the registered browser key during the encrypted nonce challenge.
- The server is the trusted server and physically controls the door relay.

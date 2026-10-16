# Harstad Makers local-only architecture diagrams

## 1) High-level system overview

```mermaid
flowchart LR
    subgraph PrivateLAN[Private local Wi‑Fi network]
        Mobile[Member mobile browser]
        PiSite[Pi-hosted mobile web site]
        API[Local Rust API on Raspberry Pi]
        Pi[Raspberry Pi door controller]
        Lock[Door lock / relay / actuator]
        WiFi[Wi‑Fi SSID: Harstad Makers]
    end

    Mobile -->|connect only on correct Wi‑Fi| WiFi
    WiFi --> PiSite
    WiFi --> API
    PiSite -->|same local network| API
    API -->|challenge + verify nonce| Mobile
    API -->|approve unlock| Pi
    Pi --> Lock
```

## 2) Local-only deployment

```mermaid
flowchart TB
    subgraph PrivateNetwork[Private network only]
        MobileBrowser[Member mobile browser]
        PiWebSite[Raspberry Pi local web site]
        RustAPI[Local Rust API]
        MemberStore[Member data + public keys]
        DoorPi[Raspberry Pi controller]
        DoorHardware[Door relay / lock]
    end

    MobileBrowser -->|connect to private SSID| PiWebSite
    PiWebSite -->|serve the member UI| MobileBrowser
    MobileBrowser -->|challenge flow| RustAPI
    RustAPI --> MemberStore
    RustAPI -->|unlock command| DoorPi
    DoorPi --> DoorHardware
```

## 3) Member mobile web flow

```mermaid
sequenceDiagram
    participant User as Member mobile browser
    participant PiSite as Pi-hosted mobile website
    participant LAN as Local Wi‑Fi network
    participant API as Local Rust API
    participant Data as Member data store
    participant DoorPi as Raspberry Pi controller
    participant Lock as Door lock

    User->>LAN: Connect to Harstad Makers Wi‑Fi
    LAN-->>User: SSID confirmed
    User->>PiSite: Open local site (raspberrypi.local)
    PiSite-->>User: Member login / access page
    User->>API: POST /api/challenge { member_id, wifi_ssid }
    API->>Data: Lookup member public key
    Data-->>API: Registered key
    API->>API: Generate nonce and encrypt with public key
    API-->>User: challenge_id + encrypted_nonce
    User->>User: Decrypt with member private key
    User->>API: POST /api/door/unlock { member_id, challenge_id, decrypted_nonce, wifi_ssid }
    API->>API: Verify SSID + nonce + member identity
    API->>DoorPi: Approve unlock
    DoorPi->>Lock: Trigger relay / unlock
    Lock-->>DoorPi: State update
    DoorPi-->>API: success
    API-->>User: approved: true
```

## 4) Internal component responsibilities

```mermaid
flowchart LR
    subgraph PiHost[Pi host]
        WebUI[Mobile web interface]
        RustAPI[Local Rust API]
        DB[Member records + public keys]
        Controller[GPIO / relay control]
    end

    subgraph AccessFlow[Access decision]
        SSID[Check Wi‑Fi SSID]
        Challenge[Generate encrypted challenge]
        Verify[Verify decrypted nonce]
        Allow[Allow unlock]
    end

    WebUI --> Challenge
    Challenge --> DB
    Verify --> Allow
    SSID --> Allow
    Allow --> Controller
    Controller --> Door[Physical lock]
```

## 5) Recommended local-only architecture

```mermaid
flowchart TB
    subgraph LocalWiFi[Local Wi‑Fi]
        Mobile[Member mobile browser]
        PiSite[Raspberry Pi site]
        API[Rust API]
        MemberData[Members + public keys]
        PiController[Door Pi relay controller]
        Lock[Door hardware]
    end

    Mobile --> PiSite
    Mobile -->|private Wi‑Fi only| API
    API --> MemberData
    API -->|validated unlock request| PiController
    PiController --> Lock
```

## 6) Security boundary

```mermaid
flowchart LR
    LocalWiFi[Private Wi‑Fi network]
    PiSite[Pi-hosted mobile website]
    RustAPI[Local Rust API]
    Door[Door hardware]

    LocalWiFi --> PiSite
    LocalWiFi --> RustAPI
    RustAPI --> Door

    classDef secure fill:#d9f7be,stroke:#389e0d,stroke-width:2px;
    class PiSite,RustAPI,Door secure;
```

## Notes

- The member website runs locally on the Raspberry Pi.
- The mobile client must be on the correct private Wi‑Fi SSID before access is allowed.
- The Raspberry Pi is both the web host and the door controller for a local-only access system.

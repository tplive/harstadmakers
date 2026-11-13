# TTL Security Threat Assessment

## Scope

This assessment reviews the TTL access-control solution as implemented in the repository, with focus on the browser flow, backend validation, Wi‑Fi trust assumptions, and local-network attack surface.

Relevant code paths reviewed:

- [ttl/public/js/ttl-crypto.mjs](ttl/public/js/ttl-crypto.mjs)
- [ttl/public/js/challenge.mjs](ttl/public/js/challenge.mjs)
- [ttl/public/js/unlock.mjs](ttl/public/js/unlock.mjs)
- [ttl/public/js/registration.mjs](ttl/public/js/registration.mjs)
- [ttl/backend/src/state.rs](ttl/backend/src/state.rs)
- [ttl/backend/src/web.rs](ttl/backend/src/web.rs)
- [ttl/backend/src/crypto.rs](ttl/backend/src/crypto.rs)

## Executive summary

The solution uses Ed25519 challenge/response and is not internet-exposed, which raises the bar against opportunistic external attackers. However, the primary security weakness is that the member’s private key is stored in the browser’s localStorage, which is not a secure key vault. A local attacker who gains access to a valid member browser or any script execution on the origin can effectively impersonate that member.

The design is strongest against internet scanning and unauthenticated outsiders. It is weakest against compromised trusted clients and local LAN attackers who can obtain valid browser state or member credentials.

## Assets under protection

- Door unlock authorization
- Member registration and activation workflow
- Server identity and challenge verification
- Local network trust assumptions
- Admin-controlled activation flow

## Threat model

### Trust boundaries

1. Browser to server boundary
2. Server to member key material
3. Local Wi‑Fi network boundary
4. Admin activation boundary outside the application
5. Client-side browser storage boundary

### Threat actors

- Remote internet attacker
- Local LAN attacker on the same Wi‑Fi
- Malicious insider or member
- Malicious browser extension or malware on a member device
- Script injection on the app origin

## Main findings

### 1. Browser localStorage is a weak key store

The member private key is stored using `localStorage` in [ttl/public/js/ttl-crypto.mjs](ttl/public/js/ttl-crypto.mjs). This means the private key is accessible to any script running on the same origin and is not protected by a hardware-backed secure enclave or dedicated secure key storage.

This is a material risk because the system’s security relies on the private key remaining private.

Risk level: High

Impact: A compromised browser or same-origin script execution can generate valid signatures and unlock the door.

### 2. Challenge-response depends on client-held secrets

The backend validates the member’s signature against the stored public key in [ttl/backend/src/state.rs](ttl/backend/src/state.rs). This is cryptographically sound if the private key remains secret. The problem is that the private key is in the browser, not in a protected device-bound environment.

Risk level: High

Impact: If an attacker obtains the private key they can impersonate the member without interacting with the server’s crypto layer.

### 3. Registration flow is too easy to abuse

The registration endpoint accepts new members in [ttl/backend/src/state.rs](ttl/backend/src/state.rs) without a direct access control gate inside the application. This is mitigated by the external admin activation process, but it still creates a flood, spoofing, and admin burden problem.

Risk level: Medium

Impact: Attackers can create many registrations and trigger manual review, causing operational disruption or social-engineering pressure on the admin.

### 4. LAN-only deployment is not a security boundary

The service runs only on the local Wi‑Fi and resolves through private LAN IPs. This reduces internet exposure but does not protect against a compromised or malicious client already on the same network.

Risk level: Medium

Impact: Anyone on the LAN can attempt the same browser logic as a valid member if they have valid credentials or client-side state.

### 5. CORS is permissive

The backend in [ttl/backend/src/web.rs](ttl/backend/src/web.rs) uses a permissive CORS policy with `allow_origin(Any)` and `allow_headers(Any)`. This is not directly sufficient to bypass the backend, but it increases the browser attack surface and broadens the risk of cross-origin abuse in a hostile web context.

Risk level: Medium

Impact: Unintended cross-origin requests can be more easily triggered from malicious pages or scripts.

## Threat scenarios

### Scenario A: Compromised member browser

1. An attacker compromises a member’s phone or browser.
2. The attacker reads the key from localStorage.
3. The attacker runs the challenge flow from their own machine on the LAN.
4. The backend verifies the signature and approves unlock.

Outcome: Door opens.

### Scenario B: Same-origin script injection

1. Malicious script executes on the TTL origin.
2. Script reads member private key and member id from browser storage.
3. Script signs the expected challenge and submits the unlock request.
4. Server accepts it as valid.

Outcome: Door opens.

### Scenario C: Local network attacker with a valid browser state

1. Attacker joins Wi‑Fi and obtains a valid member’s browser state or stolen key material.
2. Attacker calls the same API sequence as a legitimate browser.
3. Server validates the signature and allows the request.

Outcome: Door opens.

### Scenario D: Registration abuse and admin disruption

1. Attacker creates many fake registrations.
2. Admin review queue is overloaded.
3. Legitimate members are delayed or overlooked.
4. Attackers continue probing or exploit trust gaps.

Outcome: Service disruption and operational risk.

## Security posture summary

### Strong points

- Ed25519 challenge/response is a sound cryptographic primitive.
- Server-side verification is conceptually correct.
- The service is not internet-exposed.
- Manual admin activation is a useful human gate.
- Packet throttling and local network isolation reduce opportunistic abuse.

### Weak points

- The member private key is stored in the browser, not in a hardened secure key store.
- Browser-origin compromise is sufficient to steal the private key.
- The app assumes a trusted client environment, which is not strong enough for a physical door access system.
- Local Wi‑Fi access is effectively equivalent to local service access for attacker capability.

## Risk rating

Overall risk: Medium to High for the real-world deployment model.

The system is reasonably hardened against internet attacks, but it is not robust against a local attacker who can access a valid member’s key material or a compromised browser. The main risk is not cryptographic failure; it is client-side key theft and weak browser trust assumptions.

## Recommended priority improvements

1. Move private key custody out of browser localStorage and into a stronger hardware-backed or OS-level secure storage mechanism if possible.
2. Reduce the trust placed in the browser by requiring stronger device attestation or a second factor.
3. Treat LAN access as hostile by default; do not assume Wi‑Fi membership equals trust.
4. Add stronger anti-abuse controls around member creation and activation.
5. Audit whether challenge responses are strictly one-time and replay-resistant under all relevant conditions.
6. Review the browser app for scripts, extensions, and XSS exposure paths.

## Conclusion

This solution’s most significant risk is not that the Ed25519 cryptography is broken. It is that the system trusts a browser-held secret to authorize physical access. Once that secret is exposed or the browser is compromised, the door can be opened from the LAN without direct access to the server.

That means the operating assumption should be: local Wi‑Fi access plus a valid browser state equals a valid door-unlock client. That is a much weaker trust model than a hardened access-control system should aim for.

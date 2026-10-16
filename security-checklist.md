# Door Access Security Checklist

## Scope and operating model

- [ ] Remove the admin concept from the API.
- [ ] Treat admin approval as a manual server-side action on the Raspberry Pi.
- [ ] Require operators to log into the RPI and edit `members.json` directly for approvals.
- [ ] Require operators to log into the RPI and edit `members.json` directly to grant or revoke admin status.
- [ ] Document that the backend server is trusted and physically controls the door strike.

## Authentication and authorization

- [ ] Require possession of the registered private key for all member actions.
- [ ] Bind each member account to a single public key stored in `members.json`.
- [ ] Reject any unlock attempt that does not prove possession of the matching private key.
- [ ] Remove any API-driven path for promoting users to admin.
- [ ] Remove any API-driven path for approving pending members.

## Challenge and unlock flow

- [ ] Use a server-generated one-time challenge for every unlock attempt.
- [ ] Enforce a short expiration time on challenges.
- [ ] Reject expired challenges during unlock.
- [ ] Reject reused challenges after one successful unlock.
- [ ] Verify that the challenge belongs to the requesting member.
- [ ] Verify that the returned nonce exactly matches the stored challenge nonce.

## Network and browser exposure

- [ ] Restrict the service to the private Wi-Fi network.
- [ ] Limit API access to trusted mobile devices on that network.
- [ ] Remove wildcard CORS policies.
- [ ] Allow only the specific mobile app origin if browser access is required.
- [ ] Prefer direct app-to-server traffic over generic browser-based access.
- [ ] Add rate limiting to registration, challenge, and unlock endpoints.

## Data handling

- [ ] Keep `members.json` and `door_events.json` protected on the RPI.
- [ ] Restrict file permissions so only the service operator can edit them.
- [ ] Back up `members.json` before manual edits.
- [ ] Validate JSON after every manual edit.
- [ ] Keep a change log of manual membership and admin updates.

## Logging and secrets

- [ ] Stop printing private keys or sensitive registration material to stdout.
- [ ] Log registration, approval, challenge, and unlock events.
- [ ] Include timestamps and the target member ID in audit logs.
- [ ] Monitor failed unlocks and repeated challenge requests.
- [ ] Keep SSH keys and other server identity material on the RPI only.

## Host hardening

- [ ] Harden the RPI operating system.
- [ ] Patch the OS and runtime regularly.
- [ ] Limit shell access to trusted operators.
- [ ] Disable unnecessary services on the RPI.
- [ ] Keep the door-strike control path physically attached to the trusted server.
- [ ] Add a manual physical override for the door strike.

## Recovery and incident response

- [ ] Define how to revoke a compromised member key.
- [ ] Define how to disable unlocks quickly during an incident.
- [ ] Define how to restore `members.json` from backup.
- [ ] Test the full manual approval and revocation process.
- [ ] Review the audit trail after every incident.

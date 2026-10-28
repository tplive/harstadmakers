use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

const CHALLENGE_EXPIRY_MINUTES: i64 = 5;

use crate::{
    crypto::{
        create_demo_member, normalize_public_key_pem, random_nonce, verify_member_signature,
        ServerIdentity,
    },
    door_lock::{DoorLock, MockDoorLock},
    models::{
        ChallengeRecord, ChallengeRequest, ChallengeResponse, DoorEvent, Member, NewMemberRequest,
        RegistrationResponse, UnlockRequest, UnlockResponse,
    },
    persistence::{EventRepository, MemberRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub members: Arc<RwLock<Vec<Member>>>,
    pub challenges: Arc<RwLock<HashMap<String, ChallengeRecord>>>,
    pub events: Arc<RwLock<Vec<DoorEvent>>>,
    server_identity: ServerIdentity,
    member_repository: Arc<dyn MemberRepository>,
    event_repository: Arc<dyn EventRepository>,
    /// Door lock implementation - default is MockDoorLock for development
    pub door_lock: Arc<dyn DoorLock>,
}

impl AppState {
    pub fn new(
        server_identity: ServerIdentity,
        repositories: crate::persistence::Repositories,
    ) -> Result<Self> {
        // Load members from repository
        let members = match repositories.members.load_all() {
            Ok(members) => members,
            Err(_error) if repositories.bootstrap_demo_member => {
                let demo = create_demo_member();
                vec![demo]
            }
            Err(error) => return Err(error),
        };

        // Load events from repository
        let events = repositories.events.load_all()?;

        let state = Self {
            members: Arc::new(RwLock::new(members)),
            challenges: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(events)),
            server_identity,
            member_repository: repositories.members.into(),
            event_repository: repositories.events.into(),
            door_lock: Arc::new(MockDoorLock::new()),
        };

        state.persist_members()?;
        state.persist_events()?;
        Ok(state)
    }

    pub fn add_member(&self, payload: NewMemberRequest) -> Result<RegistrationResponse> {
        let public_key_pem =
            normalize_public_key_pem(payload.public_key_pem, payload.public_key_jwk)
                .context("invalid Ed25519 public key. Provide an OpenSSH key or browser JWK.")?;

        let member = Member {
            id: Uuid::new_v4().to_string(),
            name: payload.name,
            email: payload.email,
            public_key_pem,
            active: false,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        let member = self.member_repository.save_member(&member)?;
        {
            let mut members = self.members.write().unwrap();
            members.push(member.clone());
        }

        Ok(RegistrationResponse {
            member,
            server_public_key: self.server_identity.public_key_openssh().to_string(),
            server_challenge_signature: self.server_identity.sign_challenge(&payload.server_challenge)?,
            note: "Store the returned server public key and verify the signature before trusting this registration response.".to_string(),
        })
    }

    pub fn list_members(&self) -> Vec<Member> {
        self.members.read().unwrap().clone()
    }

    /// Reload members from the repository to pick up external changes
    /// Merges changes from file into the in-memory cache without removing
    /// members that only exist in memory (e.g., created during tests)
    pub fn refresh_members(&self) -> Result<()> {
        let file_members = self.member_repository.load_all()?;

        let mut members = self.members.write().unwrap();

        // Update existing members with data from file and add new ones
        for file_member in file_members {
            if let Some(existing) = members.iter_mut().find(|m| m.id == file_member.id) {
                // Update existing member with file data
                existing.active = file_member.active;
                existing.name = file_member.name;
                existing.email = file_member.email;
                existing.public_key_pem = file_member.public_key_pem;
                existing.created_at = file_member.created_at;
                existing.expires_at = file_member.expires_at;
                existing.last_used = file_member.last_used;
            } else {
                // Add new member from file
                members.push(file_member);
            }
        }

        Ok(())
    }

    pub fn generate_challenge(&self, request: ChallengeRequest) -> Result<ChallengeResponse> {
        // Refresh members from file to pick up any manual changes
        self.refresh_members()?;

        let members = self.members.read().unwrap();
        let member = members
            .iter()
            .find(|item| item.id == request.member_id)
            .cloned()
            .context("member not found")?;
        verify_member_signature(
            &member.public_key_pem,
            &request.client_nonce,
            &request.client_nonce_signature,
        )
        .context("client nonce signature verification failed")?;

        let expires_at = DateTime::parse_from_rfc3339(&member.expires_at)
            .context("member expiration timestamp is invalid")?
            .with_timezone(&Utc);
        if expires_at <= Utc::now() {
            anyhow::bail!(
                "medlemsregistreringen har utløpt og er deaktivert; vennligst kontakt administrator for reaktivering"
            );
        }

        if !member.active {
            anyhow::bail!("member is inactive and cannot request a door challenge");
        }

        let (challenge_id, nonce) = {
            let mut challenges = self.challenges.write().unwrap();
            loop {
                let challenge_id = Uuid::new_v4().to_string();
                let nonce = random_nonce();
                let id_is_unique = !challenges.contains_key(&challenge_id);
                let nonce_is_unique = !challenges
                    .values()
                    .any(|challenge| challenge.nonce == nonce);

                if id_is_unique && nonce_is_unique {
                    challenges.insert(
                        challenge_id.clone(),
                        ChallengeRecord {
                            member_id: member.id.clone(),
                            nonce: nonce.clone(),
                            expires_at: (Utc::now()
                                + chrono::Duration::minutes(CHALLENGE_EXPIRY_MINUTES))
                            .timestamp(),
                        },
                    );
                    break (challenge_id, nonce);
                }
            }
        };

        Ok(ChallengeResponse {
            challenge_id,
            member_id: member.id,
            server_nonce: nonce,
            server_challenge_signature: self.server_identity.sign_challenge(&request.server_challenge)?,
            client_nonce: request.client_nonce,
            note: "Verify the server signature, sign the returned server nonce with the member private key, then send the signature back to the unlock endpoint.".to_string(),
        })
    }

    pub fn unlock_door(&self, request: UnlockRequest) -> Result<UnlockResponse> {
        // Refresh members from file to pick up any manual changes
        self.refresh_members()?;

        let member = {
            let members = self.members.read().unwrap();
            members
                .iter()
                .find(|item| item.id == request.member_id)
                .cloned()
                .context("member not found")?
        };

        if !member.active {
            self.record_failed_unlock_attempt(&member.id)?;
            anyhow::bail!("member is inactive and cannot unlock the door");
        }

        let challenge = match {
            let challenges = self.challenges.read().unwrap();
            challenges.get(&request.challenge_id).cloned()
        } {
            Some(challenge) => challenge,
            None => {
                self.record_failed_unlock_attempt(&member.id)?;
                anyhow::bail!("challenge not found or expired");
            }
        };

        if challenge.member_id != request.member_id {
            self.record_failed_unlock_attempt(&member.id)?;
            anyhow::bail!("challenge does not belong to this member");
        }

        if Utc::now().timestamp() >= challenge.expires_at {
            self.challenges
                .write()
                .unwrap()
                .remove(&request.challenge_id);
            self.record_failed_unlock_attempt(&member.id)?;
            anyhow::bail!("challenge has expired");
        }

        if verify_member_signature(
            &member.public_key_pem,
            &challenge.nonce,
            &request.server_nonce_signature,
        )
        .is_err()
        {
            self.record_failed_unlock_attempt(&member.id)?;
            anyhow::bail!("server nonce signature verification failed");
        }

        let mut challenges = self.challenges.write().unwrap();
        challenges.remove(&request.challenge_id);
        drop(challenges);

        // Activate the door lock relay
        let unlock_time = self.door_lock.unlock()?;

        {
            let mut members = self.members.write().unwrap();
            let member = members
                .iter_mut()
                .find(|item| item.id == request.member_id)
                .context("member not found")?;
            member.last_used = Some(Utc::now().to_rfc3339());
        }

        let event = DoorEvent {
            id: Uuid::new_v4().to_string(),
            member_id: request.member_id.clone(),
            action: "unlock_door".to_string(),
            timestamp: unlock_time.to_rfc3339(),
        };

        let event = self.event_repository.save_event(&event)?;
        self.events.write().unwrap().push(event);

        self.persist_members()?;

        Ok(UnlockResponse {
            approved: true,
            member_id: request.member_id,
            message: format!(
                "Door unlock approved and relay activated at {}. The Raspberry Pi has relayed the command to the lock actuator.",
                unlock_time.to_rfc3339()
            ),
            action: "unlock_door".to_string(),
        })
    }

    fn persist_members(&self) -> Result<()> {
        let members = self.members.read().unwrap();
        self.member_repository.save_all(&members)?;
        Ok(())
    }

    fn persist_events(&self) -> Result<()> {
        let events = self.events.read().unwrap();
        self.event_repository.save_all(&events)?;
        Ok(())
    }

    fn record_failed_unlock_attempt(&self, member_id: &str) -> Result<()> {
        let event = DoorEvent {
            id: Uuid::new_v4().to_string(),
            member_id: member_id.to_string(),
            action: "failed_unlock_attempt".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };
        let event = self.event_repository.save_event(&event)?;
        self.events.write().unwrap().push(event);
        Ok(())
    }

    /// Get a reference to the door lock for testing purposes
    pub fn door_lock(&self) -> Arc<dyn DoorLock> {
        self.door_lock.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::{
            generate_test_member_key, generate_test_server_identity, verify_server_signature,
        },
        persistence::FileRepositories,
    };
    use ed25519_dalek::Signer;

    fn temp_state() -> AppState {
        let temp_dir = std::env::temp_dir().join(format!("harstad-makers-test-{}", Uuid::new_v4()));
        let repositories = FileRepositories::new(temp_dir).unwrap();
        AppState::new(generate_test_server_identity(), repositories.into()).unwrap()
    }

    #[test]
    fn demo_member_data_bootstraps() {
        let member = create_demo_member();
        assert!(member.public_key_pem.starts_with("ssh-ed25519 "));
        assert!(member.active);
    }

    #[test]
    fn active_member_can_be_activated_manually() {
        let state = temp_state();

        let member = Member {
            id: "member-1".to_string(),
            name: "Pending".to_string(),
            email: "pending@harstadmakers.no".to_string(),
            public_key_pem: create_demo_member().public_key_pem,
            active: false,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        // Manually activate member (simulating manual edit of members.json)
        {
            let mut members = state.members.write().unwrap();
            let saved = members
                .iter_mut()
                .find(|item| item.id == member.id)
                .unwrap();
            saved.active = true;
        }

        {
            let members = state.members.read().unwrap();
            let saved = members.iter().find(|item| item.id == member.id).unwrap();
            assert!(saved.active);
        }
    }

    #[test]
    fn inactive_member_cannot_unlock_door() {
        let state = temp_state();

        let member_id = "inactive-member".to_string();
        let member = Member {
            id: member_id.clone(),
            name: "Inactive".to_string(),
            email: "inactive@harstadmakers.no".to_string(),
            public_key_pem: create_demo_member().public_key_pem,
            active: false,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        let challenge_id = Uuid::new_v4().to_string();
        let nonce = "inactive-test-nonce".to_string();
        {
            let mut challenges = state.challenges.write().unwrap();
            challenges.insert(
                challenge_id.clone(),
                ChallengeRecord {
                    member_id: member.id.clone(),
                    nonce: nonce.clone(),
                    expires_at: (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
                },
            );
        }

        let request = UnlockRequest {
            member_id: member.id.clone(),
            challenge_id,
            server_nonce_signature: "invalid".to_string(),
        };

        assert!(state.unlock_door(request).is_err());
        let events = state.events.read().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "failed_unlock_attempt");
        assert_eq!(events[0].member_id, member.id);
    }

    #[test]
    fn expired_inactive_member_gets_expiration_error() {
        let state = temp_state();
        let (member_key, member_public_key) = generate_test_member_key();
        let member = Member {
            id: "expired-inactive-member".to_string(),
            name: "Expired Inactive".to_string(),
            email: "expired-inactive@harstadmakers.no".to_string(),
            public_key_pem: member_public_key,
            active: false,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() - chrono::Duration::days(1)).to_rfc3339(),
            last_used: None,
        };
        state.members.write().unwrap().push(member.clone());

        let error = state
            .generate_challenge(ChallengeRequest {
                member_id: member.id,
                server_challenge: "browser-challenge".to_string(),
                client_nonce: "client".to_string(),
                client_nonce_signature: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    member_key.sign(b"client").to_bytes(),
                ),
            })
            .unwrap_err()
            .to_string();

        assert!(error.contains("expired and is deactivated"));
        assert!(error.contains("reactivation"));
    }

    #[test]
    fn challenge_response_signs_the_server_nonce() {
        let state = temp_state();
        let (member_key, member_public_key) = generate_test_member_key();
        let mut member = create_demo_member();
        member.id = "signed-member".to_string();
        member.public_key_pem = member_public_key;

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        let response = state
            .generate_challenge(ChallengeRequest {
                member_id: member.id,
                server_challenge: "browser-challenge".to_string(),
                client_nonce: "client".to_string(),
                client_nonce_signature: {
                    let signature = member_key.sign(b"client");
                    base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        signature.to_bytes(),
                    )
                },
            })
            .unwrap();

        assert!(verify_server_signature(
            &state.server_identity,
            "browser-challenge",
            &response.server_challenge_signature,
        )
        .is_ok());
    }

    #[test]
    fn each_challenge_request_creates_a_new_challenge() {
        let state = temp_state();
        let (member_key, member_public_key) = generate_test_member_key();
        let member = Member {
            id: "unique-challenge-member".to_string(),
            name: "Unique Challenge Member".to_string(),
            email: "unique-challenge@harstadmakers.no".to_string(),
            public_key_pem: member_public_key,
            active: true,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        state.members.write().unwrap().push(member.clone());

        let sign_client_nonce = |nonce: &str| {
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                member_key.sign(nonce.as_bytes()).to_bytes(),
            )
        };
        let first = state
            .generate_challenge(ChallengeRequest {
                member_id: member.id.clone(),
                server_challenge: "browser-challenge-1".to_string(),
                client_nonce: "client-nonce-1".to_string(),
                client_nonce_signature: sign_client_nonce("client-nonce-1"),
            })
            .unwrap();
        let second = state
            .generate_challenge(ChallengeRequest {
                member_id: member.id,
                server_challenge: "browser-challenge-2".to_string(),
                client_nonce: "client-nonce-2".to_string(),
                client_nonce_signature: sign_client_nonce("client-nonce-2"),
            })
            .unwrap();

        assert_ne!(first.challenge_id, second.challenge_id);
        assert_ne!(first.server_nonce, second.server_nonce);
        assert_eq!(state.challenges.read().unwrap().len(), 2);
    }

    #[test]
    fn successful_unlock_updates_member_last_used() {
        let state = temp_state();
        let (member_key, member_public_key) = generate_test_member_key();
        let member = Member {
            id: "active-member".to_string(),
            name: "Active".to_string(),
            email: "active@harstadmakers.no".to_string(),
            public_key_pem: member_public_key,
            active: true,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        let challenge_id = Uuid::new_v4().to_string();
        let nonce = "valid-unlock-nonce".to_string();
        {
            let mut challenges = state.challenges.write().unwrap();
            challenges.insert(
                challenge_id.clone(),
                ChallengeRecord {
                    member_id: member.id.clone(),
                    nonce: nonce.clone(),
                    expires_at: (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
                },
            );
        }

        let response = state
            .unlock_door(UnlockRequest {
                member_id: member.id.clone(),
                challenge_id,
                server_nonce_signature: {
                    base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        member_key.sign(nonce.as_bytes()).to_bytes(),
                    )
                },
            })
            .unwrap();

        assert!(response.approved);

        let members = state.members.read().unwrap();
        let saved = members.iter().find(|item| item.id == member.id).unwrap();
        assert!(saved.last_used.is_some());
    }

    #[test]
    fn expired_challenge_cannot_unlock_door() {
        let state = temp_state();
        let (member_key, member_public_key) = generate_test_member_key();
        let member = Member {
            id: "expired-challenge-member".to_string(),
            name: "Expired Challenge Member".to_string(),
            email: "expired@harstadmakers.no".to_string(),
            public_key_pem: member_public_key,
            active: true,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        let challenge_id = Uuid::new_v4().to_string();
        let nonce = "expired-challenge-nonce".to_string();
        {
            let mut challenges = state.challenges.write().unwrap();
            challenges.insert(
                challenge_id.clone(),
                ChallengeRecord {
                    member_id: member.id.clone(),
                    nonce: nonce.clone(),
                    expires_at: (Utc::now() - chrono::Duration::seconds(1)).timestamp(),
                },
            );
        }

        let result = state.unlock_door(UnlockRequest {
            member_id: member.id,
            challenge_id: challenge_id.clone(),
            server_nonce_signature: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                member_key.sign(nonce.as_bytes()).to_bytes(),
            ),
        });

        assert!(result.is_err());
        assert!(!state
            .door_lock()
            .was_unlocked_between(Utc::now() - chrono::Duration::minutes(1), Utc::now(),));
        assert!(!state.challenges.read().unwrap().contains_key(&challenge_id));
        let events = state.events.read().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "failed_unlock_attempt");
        assert_eq!(events[0].member_id, "expired-challenge-member");
    }

    #[test]
    fn unlock_door_activates_door_lock() {
        let state = temp_state();
        let (member_key, member_public_key) = generate_test_member_key();
        let member = Member {
            id: "door-lock-test-member".to_string(),
            name: "Door Lock Test".to_string(),
            email: "doorlock@harstadmakers.no".to_string(),
            public_key_pem: member_public_key,
            active: true,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        let challenge_id = Uuid::new_v4().to_string();
        let nonce = "door-lock-test-nonce".to_string();
        {
            let mut challenges = state.challenges.write().unwrap();
            challenges.insert(
                challenge_id.clone(),
                ChallengeRecord {
                    member_id: member.id.clone(),
                    nonce: nonce.clone(),
                    expires_at: (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
                },
            );
        }

        // Note: MockDoorLock is used by default, so we can clear through state if needed
        // For this test, we'll work with fresh state so no clearing needed

        let before_unlock = Utc::now();
        let response = state
            .unlock_door(UnlockRequest {
                member_id: member.id.clone(),
                challenge_id,
                server_nonce_signature: {
                    base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        member_key.sign(nonce.as_bytes()).to_bytes(),
                    )
                },
            })
            .unwrap();
        let after_unlock = Utc::now();

        assert!(response.approved);

        // Verify the door lock was activated
        let door_lock = state.door_lock();
        assert!(door_lock.last_unlock_time().is_some());

        let unlock_time = door_lock.last_unlock_time().unwrap();
        assert!(unlock_time >= before_unlock && unlock_time <= after_unlock);

        // Verify it was unlocked within the correct timeframe
        assert!(door_lock.was_unlocked_between(before_unlock, after_unlock));
    }

    #[test]
    fn door_lock_mock_verifies_time_window() {
        let state = temp_state();
        let (member_key, member_public_key) = generate_test_member_key();
        let member = Member {
            id: "time-window-test-member".to_string(),
            name: "Time Window Test".to_string(),
            email: "timewindow@harstadmakers.no".to_string(),
            public_key_pem: member_public_key,
            active: true,
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        let challenge_id = Uuid::new_v4().to_string();
        let nonce = "time-window-test-nonce".to_string();
        {
            let mut challenges = state.challenges.write().unwrap();
            challenges.insert(
                challenge_id.clone(),
                ChallengeRecord {
                    member_id: member.id.clone(),
                    nonce: nonce.clone(),
                    expires_at: (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
                },
            );
        }

        let window_start = Utc::now();

        let response = state
            .unlock_door(UnlockRequest {
                member_id: member.id.clone(),
                challenge_id,
                server_nonce_signature: {
                    base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        member_key.sign(nonce.as_bytes()).to_bytes(),
                    )
                },
            })
            .unwrap();

        let window_end = Utc::now();

        assert!(response.approved);

        // Verify the door lock was activated within the time window
        let door_lock = state.door_lock();
        assert!(door_lock.was_unlocked_between(window_start, window_end));

        // Verify it was NOT unlocked before the window
        assert!(!door_lock
            .was_unlocked_between(window_start - chrono::Duration::minutes(10), window_start));

        // Verify it was NOT unlocked after the window
        assert!(
            !door_lock.was_unlocked_between(window_end, window_end + chrono::Duration::minutes(10))
        );
    }
}

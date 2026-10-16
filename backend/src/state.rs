use anyhow::{Context, Result};
use chrono::Utc;
use rsa::{pkcs1::DecodeRsaPublicKey, RsaPublicKey};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

use crate::{
    crypto::{
        create_demo_member, encrypt_with_public_key, normalize_public_key_pem, random_nonce,
        ServerIdentity,
    },
    models::{
        ChallengeRecord, ChallengeRequest, ChallengeResponse, DoorEvent, Member, NewMemberRequest,
        RegistrationResponse, UnlockRequest, UnlockResponse,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub members: Arc<RwLock<Vec<Member>>>,
    pub challenges: Arc<RwLock<HashMap<String, ChallengeRecord>>>,
    pub events: Arc<RwLock<Vec<DoorEvent>>>,
    server_identity: ServerIdentity,
    members_path: PathBuf,
    events_path: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf, server_identity: ServerIdentity) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;

        let members_path = data_dir.join("members.json");
        let events_path = data_dir.join("door_events.json");

        let members = match fs::read_to_string(&members_path) {
            Ok(json) => serde_json::from_str::<Vec<Member>>(&json).unwrap_or_else(|_| {
                let demo = create_demo_member();
                vec![demo]
            }),
            Err(_) => {
                let demo = create_demo_member();
                vec![demo]
            }
        };

        let events = match fs::read_to_string(&events_path) {
            Ok(json) => serde_json::from_str::<Vec<DoorEvent>>(&json).unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        let state = Self {
            members: Arc::new(RwLock::new(members)),
            challenges: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(events)),
            server_identity,
            members_path,
            events_path,
        };

        state.persist_members()?;
        state.persist_events()?;
        Ok(state)
    }

    pub fn add_member(&self, payload: NewMemberRequest) -> Result<RegistrationResponse> {
        let public_key_pem =
            normalize_public_key_pem(payload.public_key_pem, payload.public_key_jwk)
                .context("invalid RSA public key. Provide a PKCS#1 PEM key or a browser JWK.")?;

        let public_key = RsaPublicKey::from_pkcs1_pem(&public_key_pem)
            .context("invalid RSA public key. Use a valid PKCS#1 PEM value.")?;
        let _ = public_key;

        let member = Member {
            id: Uuid::new_v4().to_string(),
            name: payload.name,
            email: payload.email,
            public_key_pem,
            active: false,
            is_admin: false,
            created_at: Utc::now().to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = self.members.write().unwrap();
            members.push(member.clone());
        }

        self.persist_members()?;
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

    pub fn approve_member_registration(
        &self,
        member_id: &str,
        _admin_member_id: &str,
    ) -> Result<Member> {
        let mut members = self.members.write().unwrap();
        let member = members
            .iter_mut()
            .find(|member| member.id == member_id)
            .context("member not found")?;

        member.active = true;
        Ok(member.clone())
    }

    pub fn generate_challenge(&self, request: ChallengeRequest) -> Result<ChallengeResponse> {
        let members = self.members.read().unwrap();
        let member = members
            .iter()
            .find(|item| item.id == request.member_id)
            .cloned()
            .context("member not found")?;

        if !member.active {
            anyhow::bail!("member is inactive and cannot request a door challenge");
        }

        let nonce = random_nonce();
        let challenge_id = Uuid::new_v4().to_string();
        let encrypted_nonce = encrypt_with_public_key(&member.public_key_pem, &nonce)?;

        let record = ChallengeRecord {
            member_id: member.id.clone(),
            nonce: nonce.clone(),
            expires_at: (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
        };

        self.challenges
            .write()
            .unwrap()
            .insert(challenge_id.clone(), record);

        Ok(ChallengeResponse {
            challenge_id,
            member_id: member.id,
            encrypted_nonce,
            server_challenge_signature: self.server_identity.sign_challenge(&request.server_challenge)?,
            note: "Verify the server signature, decrypt the payload with the member private key, then send the plaintext nonce back to the unlock endpoint.".to_string(),
        })
    }

    pub fn unlock_door(&self, request: UnlockRequest) -> Result<UnlockResponse> {
        let member = {
            let members = self.members.read().unwrap();
            members
                .iter()
                .find(|item| item.id == request.member_id)
                .cloned()
                .context("member not found")?
        };

        if !member.active {
            anyhow::bail!("member is inactive and cannot unlock the door");
        }

        let challenge = {
            let challenges = self.challenges.read().unwrap();
            challenges
                .get(&request.challenge_id)
                .cloned()
                .context("challenge not found or expired")?
        };

        if challenge.member_id != request.member_id {
            anyhow::bail!("challenge does not belong to this member");
        }

        if challenge.nonce != request.decrypted_nonce {
            anyhow::bail!("decrypted nonce did not match the original challenge");
        }

        let mut challenges = self.challenges.write().unwrap();
        challenges.remove(&request.challenge_id);
        drop(challenges);

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
            timestamp: Utc::now().to_rfc3339(),
        };

        let mut events = self.events.write().unwrap();
        events.push(event);
        drop(events);

        self.persist_members()?;
        self.persist_events()?;

        Ok(UnlockResponse {
            approved: true,
            member_id: request.member_id,
            message:
                "Door unlock approved. The Raspberry Pi can relay the command to the lock actuator."
                    .to_string(),
            action: "unlock_door".to_string(),
        })
    }

    fn persist_members(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&*self.members.read().unwrap())?;
        fs::write(&self.members_path, json)?;
        Ok(())
    }

    fn persist_events(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&*self.events.read().unwrap())?;
        fs::write(&self.events_path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{generate_test_server_identity, verify_server_signature};

    fn temp_state() -> AppState {
        let temp_dir = std::env::temp_dir().join(format!("harstad-makers-test-{}", Uuid::new_v4()));
        AppState::new(temp_dir, generate_test_server_identity()).unwrap()
    }

    #[test]
    fn demo_member_data_bootstraps() {
        let member = create_demo_member();
        assert!(member.public_key_pem.contains("BEGIN RSA PUBLIC KEY"));
        assert!(member.is_admin);
        assert!(member.active);
    }

    #[test]
    fn active_member_can_be_activated_without_admin_gating() {
        let state = temp_state();

        let member = Member {
            id: "member-1".to_string(),
            name: "Pending".to_string(),
            email: "pending@harstadmakers.no".to_string(),
            public_key_pem: create_demo_member().public_key_pem,
            active: false,
            is_admin: false,
            created_at: Utc::now().to_rfc3339(),
            last_used: None,
        };

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        assert!(state
            .approve_member_registration(&member.id, "ignored-admin")
            .is_ok());

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
            is_admin: false,
            created_at: Utc::now().to_rfc3339(),
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
            decrypted_nonce: nonce,
        };

        assert!(state.unlock_door(request).is_err());
    }

    #[test]
    fn challenge_response_signs_the_server_nonce() {
        let state = temp_state();
        let member = create_demo_member();

        {
            let mut members = state.members.write().unwrap();
            members.push(member.clone());
        }

        let response = state
            .generate_challenge(ChallengeRequest {
                member_id: member.id,
                server_challenge: "browser-challenge".to_string(),
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
    fn successful_unlock_updates_member_last_used() {
        let state = temp_state();
        let member = Member {
            id: "active-member".to_string(),
            name: "Active".to_string(),
            email: "active@harstadmakers.no".to_string(),
            public_key_pem: create_demo_member().public_key_pem,
            active: true,
            is_admin: false,
            created_at: Utc::now().to_rfc3339(),
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
                decrypted_nonce: nonce,
            })
            .unwrap();

        assert!(response.approved);

        let members = state.members.read().unwrap();
        let saved = members.iter().find(|item| item.id == member.id).unwrap();
        assert!(saved.last_used.is_some());
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub email: String,
    pub public_key_pem: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub is_admin: bool,
    pub created_at: String,
    #[serde(default)]
    pub last_used: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoorEvent {
    pub id: String,
    pub member_id: String,
    pub action: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeRecord {
    pub member_id: String,
    pub nonce: String,
    pub expires_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewMemberRequest {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub public_key_pem: Option<String>,
    #[serde(default)]
    pub public_key_jwk: Option<serde_json::Value>,
    pub server_challenge: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApproveMemberRequest {
    pub admin_member_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeRequest {
    pub member_id: String,
    pub server_challenge: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub member: Member,
    pub server_public_key: String,
    pub server_challenge_signature: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub member_id: String,
    pub encrypted_nonce: String,
    pub server_challenge_signature: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlockRequest {
    pub member_id: String,
    pub challenge_id: String,
    pub decrypted_nonce: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlockResponse {
    pub approved: bool,
    pub member_id: String,
    pub message: String,
    pub action: String,
}

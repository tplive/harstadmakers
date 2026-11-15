use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use ureq::Agent;

use super::{EventRepository, MemberRepository};
use crate::models::{DoorEvent, DoorStatistics, Member};

struct PocketBaseClient {
    client: Agent,
    base_url: String,
    authorization: Mutex<String>,
    credentials: Option<PocketBaseCredentials>,
}

#[derive(Clone)]
pub(crate) struct PocketBaseCredentials {
    pub(crate) identity: String,
    pub(crate) password: String,
}

impl PocketBaseClient {
    fn new(
        base_url: String,
        token: Option<String>,
        credentials: Option<PocketBaseCredentials>,
    ) -> Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();
        if base_url.is_empty() {
            anyhow::bail!("POCKETBASE_URL must not be empty");
        }
        let token = token.unwrap_or_default();
        if token.trim().is_empty() && credentials.is_none() {
            anyhow::bail!("PocketBase credentials or token must be configured");
        }
        let client = Self {
            client: Agent::new(),
            base_url,
            authorization: Mutex::new(token),
            credentials,
        };
        if client.credentials.is_some() {
            client.authenticate()?;
        }
        Ok(client)
    }

    fn url(&self, collection: &str) -> String {
        format!("{}/api/collections/{collection}/records", self.base_url)
    }

    fn auth_url(&self, action: &str) -> String {
        format!("{}/api/collections/users/{action}", self.base_url)
    }

    fn authorization(&self) -> Result<String> {
        Ok(self
            .authorization
            .lock()
            .map_err(|_| anyhow::anyhow!("PocketBase authorization lock poisoned"))?
            .clone())
    }

    fn authenticate(&self) -> Result<()> {
        let credentials = self
            .credentials
            .as_ref()
            .context("PocketBase credentials are not configured")?;
        let payload = json!({
            "identity": credentials.identity,
            "password": credentials.password,
        });
        let value = self
            .response(
                self.client
                    .post(&self.auth_url("auth-with-password"))
                    .set("Content-Type", "application/json")
                    .send_string(&payload.to_string()),
            )
            .context("PocketBase authentication failed")?;
        let token = value
            .get("token")
            .and_then(Value::as_str)
            .filter(|token| !token.trim().is_empty())
            .context("PocketBase authentication returned no token")?;
        *self
            .authorization
            .lock()
            .map_err(|_| anyhow::anyhow!("PocketBase authorization lock poisoned"))? =
            token.to_string();
        Ok(())
    }

    fn authenticated_request<F>(&self, send: F) -> Result<Value>
    where
        F: Fn(&str) -> Result<ureq::Response, ureq::Error>,
    {
        let token = self.authorization()?;
        match send(&token) {
            Err(ureq::Error::Status(401, _)) if self.credentials.is_some() => {
                self.authenticate()?;
                let refreshed_token = self.authorization()?;
                self.response(send(&refreshed_token))
            }
            response => self.response(response),
        }
    }

    fn response(&self, response: Result<ureq::Response, ureq::Error>) -> Result<Value> {
        let response = match response {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let body = response
                    .into_string()
                    .context("failed to read PocketBase error response")?;
                anyhow::bail!("PocketBase request failed with {status}: {body}");
            }
            Err(error) => return Err(error).context("request to PocketBase failed"),
        };
        let status = response.status();
        let body = response
            .into_string()
            .context("failed to read PocketBase response")?;
        if !(200..300).contains(&status) {
            anyhow::bail!("PocketBase request failed with {status}: {body}");
        }
        if body.trim().is_empty() {
            Ok(Value::Null)
        } else {
            serde_json::from_str(&body).context("invalid JSON returned by PocketBase")
        }
    }

    fn get_record<T: DeserializeOwned>(
        &self,
        collection: &str,
        record_id: &str,
    ) -> Result<Option<PocketBaseRecord<T>>> {
        let response = self
            .client
            .get(&format!("{}/{}", self.url(collection), record_id));
        match response.set("Authorization", &self.authorization()?).call() {
            Ok(response) => Ok(Some(serde_json::from_value(self.response(Ok(response))?)?)),
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(ureq::Error::Status(401, _)) if self.credentials.is_some() => {
                self.authenticate()?;
                let response = self
                    .client
                    .get(&format!("{}/{}", self.url(collection), record_id))
                    .set("Authorization", &self.authorization()?)
                    .call();
                match response {
                    Ok(response) => {
                        Ok(Some(serde_json::from_value(self.response(Ok(response))?)?))
                    }
                    Err(ureq::Error::Status(404, _)) => Ok(None),
                    Err(error) => Err(anyhow::Error::new(error)
                        .context("failed to find PocketBase record after reauthentication")),
                }
            }
            Err(error) => Err(anyhow::Error::new(error).context("failed to find PocketBase record")),
        }
    }

    fn list<T: DeserializeOwned>(&self, collection: &str) -> Result<Vec<PocketBaseRecord<T>>> {
        let mut page = 1;
        let mut records = Vec::new();
        loop {
            let value = self.authenticated_request(|token| {
                self.client
                    .get(&self.url(collection))
                    .query("page", &page.to_string())
                    .query("perPage", "500")
                    .set("Authorization", token)
                    .call()
            })?;
            let result: PocketBaseList<T> = serde_json::from_value(value)?;
            records.extend(result.items);
            if page >= result.total_pages {
                return Ok(records);
            }
            page += 1;
        }
    }

    fn create_record(&self, collection: &str, payload: &Value) -> Result<Value> {
        self.authenticated_request(|token| {
            self.client
                .post(&self.url(collection))
                .set("Content-Type", "application/json")
                .set("Authorization", token)
                .send_string(&payload.to_string())
        })
    }

    fn update(&self, collection: &str, record_id: &str, payload: &Value) -> Result<()> {
        self.authenticated_request(|token| {
            self.client
                .patch(&format!("{}/{}", self.url(collection), record_id))
                .set("Content-Type", "application/json")
                .set("Authorization", token)
                .send_string(&payload.to_string())
        })?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct PocketBaseRecord<T> {
    id: String,
    #[serde(flatten)]
    fields: T,
}

#[derive(Deserialize)]
struct PocketBaseList<T> {
    #[serde(rename = "totalPages")]
    total_pages: u32,
    items: Vec<PocketBaseRecord<T>>,
}

#[derive(Serialize, Deserialize)]
struct PocketBaseMember {
    name: String,
    email: String,
    public_key_pem: String,
    active: bool,
    created_at: String,
    expires_at: String,
    last_used: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PocketBaseEvent {
    action: String,
    #[serde(default)]
    outcome: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    timestamp: String,
}

pub struct PocketBaseRepositories {
    pub(crate) members: PocketBaseMemberRepository,
    pub(crate) events: PocketBaseEventRepository,
}

impl PocketBaseRepositories {
    pub fn new(
        base_url: String,
        token: Option<String>,
        credentials: Option<PocketBaseCredentials>,
    ) -> Result<Self> {
        let members_collection = std::env::var("POCKETBASE_MEMBERS_COLLECTION")
            .unwrap_or_else(|_| "members".to_string());
        let events_collection = std::env::var("POCKETBASE_EVENTS_COLLECTION")
            .unwrap_or_else(|_| "door_events".to_string());
        let client = Arc::new(PocketBaseClient::new(base_url, token, credentials)?);
        Ok(Self {
            members: PocketBaseMemberRepository {
                client: client.clone(),
                collection: members_collection,
            },
            events: PocketBaseEventRepository {
                client,
                collection: events_collection,
            },
        })
    }
}

pub(crate) struct PocketBaseMemberRepository {
    client: Arc<PocketBaseClient>,
    collection: String,
}

impl MemberRepository for PocketBaseMemberRepository {
    fn find_by_id(&self, member_id: &str) -> Result<Option<Member>> {
        Ok(self
            .client
            .get_record::<PocketBaseMember>(&self.collection, member_id)?
            .map(|record| {
                let fields = record.fields;
                Member {
                    id: record.id,
                    name: fields.name,
                    email: fields.email,
                    public_key_pem: fields.public_key_pem,
                    active: fields.active,
                    created_at: fields.created_at,
                    expires_at: fields.expires_at,
                    last_used: fields.last_used,
                }
            }))
    }

    fn save_member(&self, member: &Member) -> Result<Member> {
        let payload = json!({
            "name": member.name,
            "email": member.email,
            "public_key_pem": member.public_key_pem,
            "active": member.active,
            "expires_at": member.expires_at,
            "last_used": member.last_used,
        });
        let response = self.client.create_record(&self.collection, &payload)?;
        let record: PocketBaseRecord<PocketBaseMember> = serde_json::from_value(response)
            .context("PocketBase returned an invalid member record")?;
        Ok(Member {
            id: record.id,
            name: record.fields.name,
            email: record.fields.email,
            public_key_pem: record.fields.public_key_pem,
            active: record.fields.active,
            created_at: record.fields.created_at,
            expires_at: record.fields.expires_at,
            last_used: record.fields.last_used,
        })
    }

    fn update_member(&self, member: &Member) -> Result<()> {
        let payload = json!({
            "name": member.name,
            "email": member.email,
            "public_key_pem": member.public_key_pem,
            "active": member.active,
            "expires_at": member.expires_at,
            "last_used": member.last_used,
        });
        self.client.update(&self.collection, &member.id, &payload)
    }
}

pub(crate) struct PocketBaseEventRepository {
    client: Arc<PocketBaseClient>,
    collection: String,
}

impl EventRepository for PocketBaseEventRepository {
    fn save_event(&self, event: &DoorEvent) -> Result<DoorEvent> {
        let payload = json!({
            "action": event.action,
            "outcome": event.outcome,
            "reason": event.reason,
        });
        let response = self.client.create_record(&self.collection, &payload)?;
        let record: PocketBaseRecord<PocketBaseEvent> = serde_json::from_value(response)
            .context("PocketBase returned an invalid door event record")?;
        let action = record.fields.action.clone();
        Ok(DoorEvent {
            id: record.id,
            action: action.clone(),
            outcome: record.fields.outcome.unwrap_or_else(|| {
                if event_is_legacy_success(&action) {
                    "success".to_string()
                } else {
                    "denied".to_string()
                }
            }),
            reason: record.fields.reason,
            timestamp: record.fields.timestamp,
        })
    }

    fn statistics(&self) -> Result<DoorStatistics> {
        let events = self
            .client
            .list::<PocketBaseEvent>(&self.collection)?
            .into_iter()
            .map(|record| {
                let action = record.fields.action.clone();
                let outcome = record.fields.outcome.unwrap_or_else(|| {
                    if event_is_legacy_success(&action) {
                        "success".to_string()
                    } else {
                        "denied".to_string()
                    }
                });
                DoorEvent {
                    id: record.id,
                    action,
                    outcome,
                    reason: record.fields.reason,
                    timestamp: record.fields.timestamp,
                }
            })
            .collect::<Vec<_>>();
        super::statistics_from_events(&events)
    }
}

fn event_is_legacy_success(action: &str) -> bool {
    action == "unlock_door"
}

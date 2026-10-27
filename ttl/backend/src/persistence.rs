use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::models::{DoorEvent, Member};

/// Trait for member persistence operations
pub trait MemberRepository: Send + Sync {
    /// Load all members from the repository
    fn load_all(&self) -> Result<Vec<Member>>;
    /// Save all members to the repository
    fn save_all(&self, members: &[Member]) -> Result<()>;
}

/// Trait for door event persistence operations
pub trait EventRepository: Send + Sync {
    /// Load all door events from the repository
    fn load_all(&self) -> Result<Vec<DoorEvent>>;
    /// Save all door events to the repository
    fn save_all(&self, events: &[DoorEvent]) -> Result<()>;
}

/// File-based implementation of MemberRepository
pub struct JsonFileMemberRepository {
    file_path: PathBuf,
}

impl JsonFileMemberRepository {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl MemberRepository for JsonFileMemberRepository {
    fn load_all(&self) -> Result<Vec<Member>> {
        let json = std::fs::read_to_string(&self.file_path)?;
        Ok(serde_json::from_str(&json)?)
    }

    fn save_all(&self, members: &[Member]) -> Result<()> {
        let json = serde_json::to_string_pretty(members)?;
        std::fs::write(&self.file_path, json)?;
        Ok(())
    }
}

/// File-based implementation of EventRepository
pub struct JsonFileEventRepository {
    file_path: PathBuf,
}

impl JsonFileEventRepository {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl EventRepository for JsonFileEventRepository {
    fn load_all(&self) -> Result<Vec<DoorEvent>> {
        match std::fs::read_to_string(&self.file_path) {
            Ok(json) => Ok(serde_json::from_str(&json)?),
            Err(_) => Ok(Vec::new()),
        }
    }

    fn save_all(&self, events: &[DoorEvent]) -> Result<()> {
        let json = serde_json::to_string_pretty(events)?;
        std::fs::write(&self.file_path, json)?;
        Ok(())
    }
}

/// Combined repository struct for convenience
pub struct FileRepositories {
    pub members: JsonFileMemberRepository,
    pub events: JsonFileEventRepository,
}

impl FileRepositories {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;

        let members_path = data_dir.join("members.json");
        let events_path = data_dir.join("door_events.json");

        Ok(Self {
            members: JsonFileMemberRepository::new(members_path),
            events: JsonFileEventRepository::new(events_path),
        })
    }
}

pub struct Repositories {
    pub members: Box<dyn MemberRepository>,
    pub events: Box<dyn EventRepository>,
    pub bootstrap_demo_member: bool,
}

impl Repositories {
    pub fn from_environment(data_dir: PathBuf) -> Result<Self> {
        match std::env::var("POCKETBASE_URL") {
            Ok(url) if !url.trim().is_empty() => {
                let token = std::env::var("POCKETBASE_TOKEN")
                    .context("POCKETBASE_TOKEN must be set when POCKETBASE_URL is configured")?;
                let repositories = PocketBaseRepositories::new(url, token)?;
                Ok(repositories.into())
            }
            _ => Ok(FileRepositories::new(data_dir)?.into()),
        }
    }
}

impl From<FileRepositories> for Repositories {
    fn from(repositories: FileRepositories) -> Self {
        Self {
            members: Box::new(repositories.members),
            events: Box::new(repositories.events),
            bootstrap_demo_member: true,
        }
    }
}

struct PocketBaseClient {
    client: Client,
    base_url: String,
    token: String,
}

impl PocketBaseClient {
    fn new(base_url: String, token: String) -> Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();
        if base_url.is_empty() {
            anyhow::bail!("POCKETBASE_URL must not be empty");
        }
        Ok(Self {
            client: Client::builder().build()?,
            base_url,
            token,
        })
    }

    fn url(&self, collection: &str) -> String {
        format!("{}/api/collections/{collection}/records", self.base_url)
    }

    fn request(&self, request: reqwest::blocking::RequestBuilder) -> Result<Value> {
        let response = request
            .header(reqwest::header::AUTHORIZATION, &self.token)
            .send()
            .context("request to PocketBase failed")?;
        let status = response.status();
        let body = response
            .text()
            .context("failed to read PocketBase response")?;
        if !status.is_success() {
            anyhow::bail!("PocketBase request failed with {status}: {body}");
        }
        if body.trim().is_empty() {
            Ok(Value::Null)
        } else {
            serde_json::from_str(&body).context("invalid JSON returned by PocketBase")
        }
    }

    fn list<T: DeserializeOwned>(&self, collection: &str) -> Result<Vec<PocketBaseRecord<T>>> {
        let mut page_number = 1;
        let mut records = Vec::new();
        loop {
            let response = self.request(
                self.client
                    .get(self.url(collection))
                    .query(&[("page", page_number), ("perPage", 500)]),
            )?;
            let result: PocketBaseList<T> = serde_json::from_value(response)?;
            let last_page = result.page >= result.total_pages;
            records.extend(result.items);
            if last_page {
                return Ok(records);
            }
            page_number += 1;
        }
    }

    fn create(&self, collection: &str, payload: &Value) -> Result<()> {
        self.request(self.client.post(self.url(collection)).json(payload))?;
        Ok(())
    }

    fn update(&self, collection: &str, record_id: &str, payload: &Value) -> Result<()> {
        self.request(
            self.client
                .patch(format!("{}/{}", self.url(collection), record_id))
                .json(payload),
        )?;
        Ok(())
    }

    fn delete(&self, collection: &str, record_id: &str) -> Result<()> {
        self.request(
            self.client
                .delete(format!("{}/{}", self.url(collection), record_id)),
        )?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct PocketBaseList<T> {
    page: u32,
    #[serde(rename = "totalPages")]
    total_pages: u32,
    items: Vec<PocketBaseRecord<T>>,
}

#[derive(Deserialize)]
struct PocketBaseRecord<T> {
    id: String,
    #[serde(flatten)]
    fields: T,
}

#[derive(Serialize, Deserialize)]
struct PocketBaseMember {
    member_id: String,
    name: String,
    email: String,
    public_key_pem: String,
    active: bool,
    created_at: String,
    last_used: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PocketBaseEvent {
    event_id: String,
    member_id: String,
    action: String,
    timestamp: String,
}

pub struct PocketBaseRepositories {
    members: PocketBaseMemberRepository,
    events: PocketBaseEventRepository,
}

impl PocketBaseRepositories {
    pub fn new(base_url: String, token: String) -> Result<Self> {
        let members_collection = std::env::var("POCKETBASE_MEMBERS_COLLECTION")
            .unwrap_or_else(|_| "members".to_string());
        let events_collection = std::env::var("POCKETBASE_EVENTS_COLLECTION")
            .unwrap_or_else(|_| "door_events".to_string());
        let client = std::sync::Arc::new(PocketBaseClient::new(base_url, token)?);
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

impl From<PocketBaseRepositories> for Repositories {
    fn from(repositories: PocketBaseRepositories) -> Self {
        Self {
            members: Box::new(repositories.members),
            events: Box::new(repositories.events),
            bootstrap_demo_member: false,
        }
    }
}

struct PocketBaseMemberRepository {
    client: std::sync::Arc<PocketBaseClient>,
    collection: String,
}

impl MemberRepository for PocketBaseMemberRepository {
    fn load_all(&self) -> Result<Vec<Member>> {
        self.client
            .list::<PocketBaseMember>(&self.collection)?
            .into_iter()
            .map(|record| {
                let fields = record.fields;
                Ok(Member {
                    id: fields.member_id,
                    name: fields.name,
                    email: fields.email,
                    public_key_pem: fields.public_key_pem,
                    active: fields.active,
                    created_at: fields.created_at,
                    last_used: fields.last_used,
                })
            })
            .collect()
    }

    fn save_all(&self, members: &[Member]) -> Result<()> {
        let existing = self.client.list::<PocketBaseMember>(&self.collection)?;
        for member in members {
            let payload = json!({
                "member_id": member.id,
                "name": member.name,
                "email": member.email,
                "public_key_pem": member.public_key_pem,
                "active": member.active,
                "created_at": member.created_at,
                "last_used": member.last_used,
            });
            if let Some(record) = existing
                .iter()
                .find(|record| record.fields.member_id == member.id)
            {
                self.client.update(&self.collection, &record.id, &payload)?;
            } else {
                self.client.create(&self.collection, &payload)?;
            }
        }
        for record in existing {
            if !members
                .iter()
                .any(|member| member.id == record.fields.member_id)
            {
                self.client.delete(&self.collection, &record.id)?;
            }
        }
        Ok(())
    }
}

struct PocketBaseEventRepository {
    client: std::sync::Arc<PocketBaseClient>,
    collection: String,
}

impl EventRepository for PocketBaseEventRepository {
    fn load_all(&self) -> Result<Vec<DoorEvent>> {
        self.client
            .list::<PocketBaseEvent>(&self.collection)?
            .into_iter()
            .map(|record| {
                let fields = record.fields;
                Ok(DoorEvent {
                    id: fields.event_id,
                    member_id: fields.member_id,
                    action: fields.action,
                    timestamp: fields.timestamp,
                })
            })
            .collect()
    }

    fn save_all(&self, events: &[DoorEvent]) -> Result<()> {
        let existing = self.client.list::<PocketBaseEvent>(&self.collection)?;
        for event in events {
            let payload = json!({
                "event_id": event.id,
                "member_id": event.member_id,
                "action": event.action,
                "timestamp": event.timestamp,
            });
            if let Some(record) = existing
                .iter()
                .find(|record| record.fields.event_id == event.id)
            {
                self.client.update(&self.collection, &record.id, &payload)?;
            } else {
                self.client.create(&self.collection, &payload)?;
            }
        }
        for record in existing {
            if !events
                .iter()
                .any(|event| event.id == record.fields.event_id)
            {
                self.client.delete(&self.collection, &record.id)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::create_demo_member;
    use std::env;
    use uuid::Uuid;

    #[test]
    fn test_member_repository_roundtrip() {
        let temp_dir = env::temp_dir().join(format!("test-members-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let repo = JsonFileMemberRepository::new(temp_dir.join("members.json"));

        let members = vec![Member {
            id: "test-1".to_string(),
            name: "Test Member".to_string(),
            email: "test@harstadmakers.no".to_string(),
            public_key_pem: create_demo_member().public_key_pem,
            active: true,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            last_used: None,
        }];

        repo.save_all(&members).unwrap();
        let loaded = repo.load_all().unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test-1");
    }

    #[test]
    fn test_event_repository_roundtrip() {
        let temp_dir = env::temp_dir().join(format!("test-events-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let repo = JsonFileEventRepository::new(temp_dir.join("events.json"));

        let events = vec![DoorEvent {
            id: "event-1".to_string(),
            member_id: "member-1".to_string(),
            action: "unlock_door".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        }];

        repo.save_all(&events).unwrap();
        let loaded = repo.load_all().unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "event-1");
    }
}

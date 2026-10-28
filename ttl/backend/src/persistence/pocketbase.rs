use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use ureq::Agent;

use super::{EventRepository, MemberRepository};
use crate::models::{DoorEvent, Member};

struct PocketBaseClient {
    client: Agent,
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
            client: Agent::new(),
            base_url,
            token,
        })
    }

    fn url(&self, collection: &str) -> String {
        format!("{}/api/collections/{collection}/records", self.base_url)
    }

    fn collection_url(&self, collection: &str) -> String {
        format!("{}/api/collections/{collection}", self.base_url)
    }

    fn request(&self, request: ureq::Request) -> Result<Value> {
        self.response(request.set("Authorization", &self.token).call())
    }

    fn response(&self, response: Result<ureq::Response, ureq::Error>) -> Result<Value> {
        let response = response.context("request to PocketBase failed")?;
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

    fn list<T: DeserializeOwned>(&self, collection: &str) -> Result<Vec<PocketBaseRecord<T>>> {
        let mut page_number = 1;
        let mut records = Vec::new();
        loop {
            let response = self.request(
                self.client
                    .get(&self.url(collection))
                    .query("page", &page_number.to_string())
                    .query("perPage", "500"),
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

    fn validate_collection(
        &self,
        collection: &str,
        expected_fields: &[(&str, &str)],
    ) -> Result<PocketBaseCollection> {
        let response = self
            .request(self.client.get(&self.collection_url(collection)))
            .with_context(|| {
                format!("PocketBase collection '{collection}' was not found or is not accessible")
            })?;
        let schema: PocketBaseCollection = serde_json::from_value(response).with_context(|| {
            format!("invalid schema returned for PocketBase collection '{collection}'")
        })?;

        validate_collection_schema(collection, &schema, expected_fields)?;
        for field_name in expected_fields
            .iter()
            .filter_map(|(name, field_type)| (*field_type == "autodate").then_some(*name))
        {
            validate_autodate_field(collection, &schema, field_name)?;
        }
        Ok(schema)
    }

    fn create(&self, collection: &str, payload: &Value) -> Result<()> {
        self.create_record(collection, payload)?;
        Ok(())
    }

    fn create_record(&self, collection: &str, payload: &Value) -> Result<Value> {
        self.response(
            self.client
                .post(&self.url(collection))
                .set("Content-Type", "application/json")
                .set("Authorization", &self.token)
                .send_string(&payload.to_string()),
        )
    }

    fn update(&self, collection: &str, record_id: &str, payload: &Value) -> Result<()> {
        self.response(
            self.client
                .patch(&format!("{}/{}", self.url(collection), record_id))
                .set("Content-Type", "application/json")
                .set("Authorization", &self.token)
                .send_string(&payload.to_string()),
        )?;
        Ok(())
    }

    fn delete(&self, collection: &str, record_id: &str) -> Result<()> {
        self.response(
            self.client
                .delete(&format!("{}/{}", self.url(collection), record_id))
                .set("Authorization", &self.token)
                .call(),
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
struct PocketBaseCollection {
    id: String,
    #[serde(default)]
    fields: Vec<PocketBaseField>,
}

#[derive(Deserialize)]
struct PocketBaseField {
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    #[serde(default)]
    #[serde(rename = "onCreate")]
    on_create: bool,
    #[serde(default)]
    #[serde(rename = "onUpdate")]
    on_update: bool,
    #[serde(default)]
    #[serde(rename = "collectionId")]
    collection_id: Option<String>,
}

fn validate_collection_schema(
    collection: &str,
    schema: &PocketBaseCollection,
    expected_fields: &[(&str, &str)],
) -> Result<()> {
    for (field_name, expected_type) in expected_fields {
        let field = schema
            .fields
            .iter()
            .find(|field| field.name == *field_name)
            .with_context(|| {
                format!(
                    "PocketBase collection '{collection}' is missing required field '{field_name}'"
                )
            })?;
        if field.field_type != *expected_type {
            anyhow::bail!(
                "PocketBase collection '{collection}' field '{field_name}' has type '{}', expected '{}'",
                field.field_type,
                expected_type
            );
        }
    }
    Ok(())
}

fn validate_autodate_field(
    collection: &str,
    schema: &PocketBaseCollection,
    field_name: &str,
) -> Result<()> {
    let field = schema
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .with_context(|| {
            format!("PocketBase collection '{collection}' is missing field '{field_name}'")
        })?;
    if field.field_type != "autodate" || !field.on_create || field.on_update {
        anyhow::bail!(
            "PocketBase collection '{collection}' field '{field_name}' must be autodate with onCreate=true and onUpdate=false"
        );
    }

    Ok(())
}

fn validate_relation_field(
    collection: &str,
    schema: &PocketBaseCollection,
    field_name: &str,
    target_collection_id: &str,
) -> Result<()> {
    let field = schema
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .with_context(|| {
            format!("PocketBase collection '{collection}' is missing field '{field_name}'")
        })?;
    if field.field_type != "relation"
        || field.collection_id.as_deref() != Some(target_collection_id)
    {
        anyhow::bail!(
            "PocketBase collection '{collection}' field '{field_name}' must be a relation to collection '{target_collection_id}'"
        );
    }
    Ok(())
}

#[derive(Deserialize)]
struct PocketBaseRecord<T> {
    id: String,
    #[serde(flatten)]
    fields: T,
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
    member_id: String,
    action: String,
    timestamp: String,
}

pub struct PocketBaseRepositories {
    pub(crate) members: PocketBaseMemberRepository,
    pub(crate) events: PocketBaseEventRepository,
}

impl PocketBaseRepositories {
    pub fn new(base_url: String, token: String) -> Result<Self> {
        let members_collection = std::env::var("POCKETBASE_MEMBERS_COLLECTION")
            .unwrap_or_else(|_| "members".to_string());
        let events_collection = std::env::var("POCKETBASE_EVENTS_COLLECTION")
            .unwrap_or_else(|_| "door_events".to_string());
        let client = Arc::new(PocketBaseClient::new(base_url, token)?);
        let members_schema = client.validate_collection(
            &members_collection,
            &[
                ("name", "text"),
                ("email", "text"),
                ("public_key_pem", "text"),
                ("active", "bool"),
                ("created_at", "autodate"),
                ("expires_at", "date"),
                ("last_used", "date"),
            ],
        )?;
        let events_schema = client.validate_collection(
            &events_collection,
            &[
                ("member_id", "relation"),
                ("action", "text"),
                ("timestamp", "autodate"),
            ],
        )?;
        validate_relation_field(
            &events_collection,
            &events_schema,
            "member_id",
            &members_schema.id,
        )?;
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
    fn load_all(&self) -> Result<Vec<Member>> {
        self.client
            .list::<PocketBaseMember>(&self.collection)?
            .into_iter()
            .map(|record| {
                let fields = record.fields;
                Ok(Member {
                    id: record.id,
                    name: fields.name,
                    email: fields.email,
                    public_key_pem: fields.public_key_pem,
                    active: fields.active,
                    created_at: fields.created_at,
                    expires_at: fields.expires_at,
                    last_used: fields.last_used,
                })
            })
            .collect()
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

    fn save_all(&self, members: &[Member]) -> Result<()> {
        let existing = self.client.list::<PocketBaseMember>(&self.collection)?;
        for member in members {
            let payload = json!({
                "name": member.name,
                "email": member.email,
                "public_key_pem": member.public_key_pem,
                "active": member.active,
                "expires_at": member.expires_at,
                "last_used": member.last_used,
            });
            if let Some(record) = existing.iter().find(|record| record.id == member.id) {
                self.client.update(&self.collection, &record.id, &payload)?;
            } else {
                self.client.create(&self.collection, &payload)?;
            }
        }
        for record in existing {
            if !members.iter().any(|member| member.id == record.id) {
                self.client.delete(&self.collection, &record.id)?;
            }
        }
        Ok(())
    }
}

pub(crate) struct PocketBaseEventRepository {
    client: Arc<PocketBaseClient>,
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
                    id: record.id,
                    member_id: fields.member_id,
                    action: fields.action,
                    timestamp: fields.timestamp,
                })
            })
            .collect()
    }

    fn save_event(&self, event: &DoorEvent) -> Result<DoorEvent> {
        let payload = json!({
            "member_id": event.member_id,
            "action": event.action,
        });
        let response = self.client.create_record(&self.collection, &payload)?;
        let record: PocketBaseRecord<PocketBaseEvent> = serde_json::from_value(response)
            .context("PocketBase returned an invalid door event record")?;
        Ok(DoorEvent {
            id: record.id,
            member_id: record.fields.member_id,
            action: record.fields.action,
            timestamp: record.fields.timestamp,
        })
    }

    fn save_all(&self, events: &[DoorEvent]) -> Result<()> {
        let existing = self.client.list::<PocketBaseEvent>(&self.collection)?;
        for event in events {
            let payload = json!({
                "member_id": event.member_id,
                "action": event.action,
            });
            if let Some(record) = existing.iter().find(|record| record.id == event.id) {
                self.client.update(&self.collection, &record.id, &payload)?;
            } else {
                self.client.create(&self.collection, &payload)?;
            }
        }

        for record in existing {
            if !events.iter().any(|event| event.id == record.id) {
                self.client.delete(&self.collection, &record.id)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_collection_schema() {
        let schema: PocketBaseCollection = serde_json::from_value(serde_json::json!({
            "id": "members_collection",
            "fields": [{"name": "active", "type": "bool"}]
        }))
        .unwrap();

        assert!(validate_collection_schema("members", &schema, &[("active", "bool")]).is_ok());
        assert!(validate_collection_schema("members", &schema, &[("email", "text")]).is_err());
        assert!(validate_collection_schema("members", &schema, &[("active", "text")]).is_err());
    }

    #[test]
    fn validates_autodate_flags() {
        let schema: PocketBaseCollection = serde_json::from_value(serde_json::json!({
            "id": "members_collection",
            "fields": [{
                "name": "created_at",
                "type": "autodate",
                "onCreate": true,
                "onUpdate": false
            }]
        }))
        .unwrap();

        assert!(validate_autodate_field("members", &schema, "created_at").is_ok());
    }

    #[test]
    fn validates_relation_target() {
        let schema: PocketBaseCollection = serde_json::from_value(serde_json::json!({
            "id": "events_collection",
            "fields": [{
                "name": "member_id",
                "type": "relation",
                "collectionId": "members_collection"
            }]
        }))
        .unwrap();

        assert!(
            validate_relation_field("door_events", &schema, "member_id", "members_collection")
                .is_ok()
        );
        assert!(
            validate_relation_field("door_events", &schema, "member_id", "other_collection")
                .is_err()
        );
    }
}

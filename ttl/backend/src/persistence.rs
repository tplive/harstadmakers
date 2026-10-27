use anyhow::Result;
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

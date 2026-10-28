mod json_file;
mod pocketbase;

#[allow(unused_imports)]
pub use json_file::{FileRepositories, JsonFileEventRepository, JsonFileMemberRepository};
pub use pocketbase::PocketBaseRepositories;

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::models::{DoorEvent, Member};

/// Trait for member persistence operations.
pub trait MemberRepository: Send + Sync {
    /// Load all members from the repository.
    fn load_all(&self) -> Result<Vec<Member>>;
    /// Create or persist one member and return its repository-assigned identity.
    fn save_member(&self, member: &Member) -> Result<Member>;
    /// Save all members to the repository.
    fn save_all(&self, members: &[Member]) -> Result<()>;
}

/// Trait for door event persistence operations.
pub trait EventRepository: Send + Sync {
    /// Load all door events from the repository.
    fn load_all(&self) -> Result<Vec<DoorEvent>>;
    /// Create one event and return its repository-assigned identity.
    fn save_event(&self, event: &DoorEvent) -> Result<DoorEvent>;
    /// Save all door events to the repository.
    fn save_all(&self, events: &[DoorEvent]) -> Result<()>;
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
                Ok(PocketBaseRepositories::new(url, token)?.into())
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

impl From<PocketBaseRepositories> for Repositories {
    fn from(repositories: PocketBaseRepositories) -> Self {
        Self {
            members: Box::new(repositories.members),
            events: Box::new(repositories.events),
            bootstrap_demo_member: false,
        }
    }
}

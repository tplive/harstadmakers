mod json_file;
mod pocketbase;

#[allow(unused_imports)]
pub use json_file::{FileRepositories, JsonFileEventRepository, JsonFileMemberRepository};
pub use pocketbase::PocketBaseRepositories;

use anyhow::Result;

use crate::configuration::Configuration;
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
    pub fn from_configuration(configuration: &Configuration) -> Result<Self> {
        if !configuration.pocketbase_url.trim().is_empty() {
            let token = configuration.pocketbase_token.trim();
            if token.is_empty() {
                anyhow::bail!("pocketbase_token must be set when pocketbase_url is configured");
            }
            if token == "YOUR_POCKETBASE_TOKEN" {
                anyhow::bail!(
                    "pocketbase_token still contains the configuration placeholder; set a valid PocketBase token"
                );
            }
            return Ok(PocketBaseRepositories::new(
                configuration.pocketbase_url.clone(),
                token.to_string(),
            )?
            .into());
        }

        Ok(FileRepositories::new(configuration.data_dir.clone())?.into())
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

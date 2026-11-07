mod pocketbase;

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
}

impl Repositories {
    pub fn from_configuration(configuration: &Configuration) -> Result<Self> {
        let token = configuration.pocketbase_token.trim();
        if token.is_empty() {
            anyhow::bail!("pocketbase_token must be set");
        }
        if token == "YOUR_POCKETBASE_TOKEN" || token == "YOUR_POCKETBASE_USER_TOKEN_HERE" {
            anyhow::bail!(
                "pocketbase_token still contains the configuration placeholder; set a valid PocketBase token"
            );
        }
        Ok(PocketBaseRepositories::new(
            configuration.pocketbase_url.clone(),
            token.to_string(),
        )?
        .into())
    }
}

impl From<PocketBaseRepositories> for Repositories {
    fn from(repositories: PocketBaseRepositories) -> Self {
        Self {
            members: Box::new(repositories.members),
            events: Box::new(repositories.events),
        }
    }
}

#[cfg(test)]
pub struct InMemoryRepositories {
    pub members: std::sync::Mutex<Vec<Member>>,
    pub events: std::sync::Mutex<Vec<DoorEvent>>,
}

#[cfg(test)]
impl InMemoryRepositories {
    pub fn new() -> Self {
        Self {
            members: std::sync::Mutex::new(Vec::new()),
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[cfg(test)]
impl MemberRepository for std::sync::Arc<InMemoryRepositories> {
    fn load_all(&self) -> Result<Vec<Member>> {
        Ok(self.members.lock().unwrap().clone())
    }

    fn save_member(&self, member: &Member) -> Result<Member> {
        self.members.lock().unwrap().push(member.clone());
        Ok(member.clone())
    }

    fn save_all(&self, members: &[Member]) -> Result<()> {
        *self.members.lock().unwrap() = members.to_vec();
        Ok(())
    }
}

#[cfg(test)]
impl EventRepository for std::sync::Arc<InMemoryRepositories> {
    fn load_all(&self) -> Result<Vec<DoorEvent>> {
        Ok(self.events.lock().unwrap().clone())
    }

    fn save_event(&self, event: &DoorEvent) -> Result<DoorEvent> {
        self.events.lock().unwrap().push(event.clone());
        Ok(event.clone())
    }

    fn save_all(&self, events: &[DoorEvent]) -> Result<()> {
        *self.events.lock().unwrap() = events.to_vec();
        Ok(())
    }
}

#[cfg(test)]
impl From<std::sync::Arc<InMemoryRepositories>> for Repositories {
    fn from(repositories: std::sync::Arc<InMemoryRepositories>) -> Self {
        Self {
            members: Box::new(repositories.clone()),
            events: Box::new(repositories),
        }
    }
}

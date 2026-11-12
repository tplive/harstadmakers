mod pocketbase;

pub use pocketbase::PocketBaseRepositories;

use anyhow::Result;

use crate::configuration::Configuration;
use crate::models::{CountByLabel, DoorEvent, DoorStatistics, Member};

/// Trait for member persistence operations.
pub trait MemberRepository: Send + Sync {
    /// Load one member by its repository ID.
    fn find_by_id(&self, member_id: &str) -> Result<Option<Member>>;
    /// Create or persist one member and return its repository-assigned identity.
    fn save_member(&self, member: &Member) -> Result<Member>;
    /// Update one existing member.
    fn update_member(&self, member: &Member) -> Result<()>;
}

/// Trait for door event persistence operations.
pub trait EventRepository: Send + Sync {
    /// Create one event and return its repository-assigned identity.
    fn save_event(&self, event: &DoorEvent) -> Result<DoorEvent>;
    /// Return aggregate statistics without exposing raw event records.
    fn statistics(&self) -> Result<DoorStatistics>;
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
    fn find_by_id(&self, member_id: &str) -> Result<Option<Member>> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .iter()
            .find(|member| member.id == member_id)
            .cloned())
    }

    fn save_member(&self, member: &Member) -> Result<Member> {
        self.members.lock().unwrap().push(member.clone());
        Ok(member.clone())
    }

    fn update_member(&self, member: &Member) -> Result<()> {
        if let Some(existing) = self
            .members
            .lock()
            .unwrap()
            .iter_mut()
            .find(|existing| existing.id == member.id)
        {
            *existing = member.clone();
        }
        Ok(())
    }
}

#[cfg(test)]
impl EventRepository for std::sync::Arc<InMemoryRepositories> {
    fn save_event(&self, event: &DoorEvent) -> Result<DoorEvent> {
        self.events.lock().unwrap().push(event.clone());
        Ok(event.clone())
    }

    fn statistics(&self) -> Result<DoorStatistics> {
        statistics_from_events(&self.events.lock().unwrap())
    }

}

fn statistics_from_events(events: &[DoorEvent]) -> Result<DoorStatistics> {
    use chrono::{DateTime, Datelike, Timelike, Utc};
    use std::collections::BTreeMap;
    let mut result = DoorStatistics::default();
    let mut hours = [0_u64; 24];
    let mut weekdays = [0_u64; 7];
    let mut monthly = BTreeMap::new();
    let mut yearly = BTreeMap::new();
    let mut denied_by_reason = BTreeMap::new();
    for event in events {
        let timestamp = event.timestamp.parse::<DateTime<Utc>>()?;
        let outcome = if event.outcome.is_empty() {
            if event.action == "unlock_door" { "success" } else { "denied" }
        } else {
            event.outcome.as_str()
        };
        let bucket = if outcome == "success" {
            result.successful_unlocks += 1;
            &mut hours[timestamp.hour() as usize]
        } else if outcome == "relay_failure" {
            result.relay_failures += 1;
            continue;
        } else {
            result.failed_attempts += 1;
            let reason = event.reason.as_deref().unwrap_or("unknown");
            *denied_by_reason.entry(reason.to_string()).or_insert(0) += 1;
            continue;
        };
        *bucket += 1;
        weekdays[timestamp.weekday().num_days_from_monday() as usize] += 1;
        *monthly.entry(timestamp.format("%Y-%m").to_string()).or_insert(0) += 1;
        *yearly.entry(timestamp.year().to_string()).or_insert(0) += 1;
    }
    result.by_hour = hours.iter().enumerate().map(|(i, count)| CountByLabel { label: format!("{i:02}"), count: *count }).collect();
    result.by_weekday = weekdays.iter().enumerate().map(|(i, count)| CountByLabel { label: i.to_string(), count: *count }).collect();
    result.monthly = monthly.into_iter().map(|(label, count)| CountByLabel { label, count }).collect();
    result.yearly = yearly.into_iter().map(|(label, count)| CountByLabel { label, count }).collect();
    result.denied_by_reason = denied_by_reason.into_iter().map(|(label, count)| CountByLabel { label, count }).collect();
    Ok(result)
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

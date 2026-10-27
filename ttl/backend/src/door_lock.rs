//! Door lock module with relay control
//!
//! This module provides a trait-based abstraction for door lock operations,
//! allowing for both mock implementations (for testing) and hardware implementations.

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

/// Trait for door lock operations
pub trait DoorLock: Send + Sync {
    /// Activate the door lock relay to unlock the door
    /// Returns the timestamp when the unlock was activated
    fn unlock(&self) -> Result<DateTime<Utc>>;

    /// Check if the door lock was activated within a specific time window
    /// Used for testing to verify that unlock happened in the correct timeframe
    fn was_unlocked_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool;

    /// Get the last unlock timestamp
    fn last_unlock_time(&self) -> Option<DateTime<Utc>>;
}

/// Mock door lock implementation for testing and development
/// 
/// This mock tracks when unlock was called and can verify that
/// an approved unlocking activates the door lock within the correct timeframe.
#[derive(Debug, Clone)]
pub struct MockDoorLock {
    /// Timestamps of all unlock operations
    unlock_timestamps: Arc<Mutex<Vec<DateTime<Utc>>>>,
}

impl MockDoorLock {
    /// Create a new mock door lock
    pub fn new() -> Self {
        Self {
            unlock_timestamps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all unlock timestamps (for testing purposes)
    pub fn unlock_timestamps(&self) -> Vec<DateTime<Utc>> {
        self.unlock_timestamps.lock().unwrap().clone()
    }

    /// Clear all unlock timestamps (for testing purposes)
    pub fn clear(&self) {
        self.unlock_timestamps.lock().unwrap().clear();
    }
}

impl Default for MockDoorLock {
    fn default() -> Self {
        Self::new()
    }
}

impl DoorLock for MockDoorLock {
    fn unlock(&self) -> Result<DateTime<Utc>> {
        let now = Utc::now();
        self.unlock_timestamps.lock().unwrap().push(now);
        Ok(now)
    }

    fn was_unlocked_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
        let timestamps = self.unlock_timestamps.lock().unwrap();
        timestamps.iter().any(|&ts| ts >= start && ts <= end)
    }

    fn last_unlock_time(&self) -> Option<DateTime<Utc>> {
        let timestamps = self.unlock_timestamps.lock().unwrap();
        timestamps.last().cloned()
    }
}

/// Hardware door lock implementation using GPIO relay
/// 
/// This would be the production implementation that controls
/// the actual relay hardware connected to the door lock.
/// Currently a placeholder - to be implemented when hardware is ready.
#[derive(Debug, Clone)]
pub struct HardwareDoorLock;

impl DoorLock for HardwareDoorLock {
    fn unlock(&self) -> Result<DateTime<Utc>> {
        // TODO: Implement actual GPIO relay control
        // For now, just log and return current time
        println!("HardwareDoorLock: Activating relay to unlock door");
        // In production, this would:
        // 1. Set GPIO pin high to activate relay
        // 2. Wait for the configured unlock duration
        // 3. Set GPIO pin low to deactivate relay
        Ok(Utc::now())
    }

    fn was_unlocked_between(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> bool {
        // Hardware implementation cannot track history by default
        // This would require additional logging infrastructure
        false
    }

    fn last_unlock_time(&self) -> Option<DateTime<Utc>> {
        // Hardware implementation cannot track history by default
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_door_lock_records_unlock() {
        let mock = MockDoorLock::new();
        
        let before = Utc::now();
        let result = mock.unlock().unwrap();
        let after = Utc::now();
        
        assert!(result >= before && result <= after);
        assert_eq!(mock.unlock_timestamps().len(), 1);
        assert_eq!(mock.last_unlock_time(), Some(result));
    }

    #[test]
    fn mock_door_lock_tracks_multiple_unlocks() {
        let mock = MockDoorLock::new();
        
        mock.unlock().unwrap();
        mock.unlock().unwrap();
        mock.unlock().unwrap();
        
        assert_eq!(mock.unlock_timestamps().len(), 3);
    }

    #[test]
    fn mock_door_lock_clear() {
        let mock = MockDoorLock::new();
        
        mock.unlock().unwrap();
        assert_eq!(mock.unlock_timestamps().len(), 1);
        
        mock.clear();
        assert_eq!(mock.unlock_timestamps().len(), 0);
    }

    #[test]
    fn mock_door_lock_was_unlocked_between() {
        let mock = MockDoorLock::new();
        
        let start = Utc::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        mock.unlock().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let end = Utc::now();
        
        assert!(mock.was_unlocked_between(start, end));
        assert!(!mock.was_unlocked_between(end, end + chrono::Duration::seconds(1)));
    }



    #[test]
    fn hardware_door_lock_unlock() {
        let hw = HardwareDoorLock;
        let result = hw.unlock().unwrap();
        
        let now = Utc::now();
        assert!(result >= (now - chrono::Duration::seconds(1)) && result <= now);
    }
}

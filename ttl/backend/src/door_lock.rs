//! Door lock implementations backed by either GPIO hardware or an in-memory mock.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub trait DoorLock: Send + Sync {
    fn unlock(&self) -> Result<DateTime<Utc>>;
    fn was_unlocked_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool;
    fn last_unlock_time(&self) -> Option<DateTime<Utc>>;
    fn is_door_open(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct MockDoorLock {
    unlock_timestamps: Arc<Mutex<Vec<DateTime<Utc>>>>,
    door_open: Arc<Mutex<bool>>,
}

impl MockDoorLock {
    pub fn new() -> Self {
        Self {
            unlock_timestamps: Arc::new(Mutex::new(Vec::new())),
            door_open: Arc::new(Mutex::new(false)),
        }
    }

    pub fn unlock_timestamps(&self) -> Vec<DateTime<Utc>> {
        self.unlock_timestamps.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.unlock_timestamps.lock().unwrap().clear();
    }

    pub fn set_door_open(&self, door_open: bool) {
        *self.door_open.lock().unwrap() = door_open;
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
        self.unlock_timestamps
            .lock()
            .unwrap()
            .iter()
            .any(|&timestamp| timestamp >= start && timestamp <= end)
    }

    fn last_unlock_time(&self) -> Option<DateTime<Utc>> {
        self.unlock_timestamps.lock().unwrap().last().cloned()
    }

    fn is_door_open(&self) -> bool {
        *self.door_open.lock().unwrap()
    }
}

#[derive(Debug)]
pub struct HardwareDoorLock {
    relay: Mutex<OutputPin>,
    hall_sensor: Mutex<InputPin>,
    unlock_duration: Duration,
}

impl HardwareDoorLock {
    pub fn new(
        relay_gpio_pin: u8,
        hall_sensor_gpio_pin: u8,
        unlock_duration: Duration,
    ) -> Result<Self> {
        let gpio = Gpio::new()?;
        let relay = gpio.get(relay_gpio_pin)?.into_output_low();
        let hall_sensor = gpio.get(hall_sensor_gpio_pin)?.into_input_pullup();

        Ok(Self {
            relay: Mutex::new(relay),
            hall_sensor: Mutex::new(hall_sensor),
            unlock_duration,
        })
    }
}

impl DoorLock for HardwareDoorLock {
    fn unlock(&self) -> Result<DateTime<Utc>> {
        let mut relay = self.relay.lock().unwrap();
        relay.set_high();
        std::thread::sleep(self.unlock_duration);
        relay.set_low();
        Ok(Utc::now())
    }

    fn was_unlocked_between(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> bool {
        false
    }

    fn last_unlock_time(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn is_door_open(&self) -> bool {
        self.hall_sensor.lock().unwrap().is_low()
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
        mock.clear();
        assert!(mock.unlock_timestamps().is_empty());
    }

    #[test]
    fn mock_door_lock_was_unlocked_between() {
        let mock = MockDoorLock::new();
        let start = Utc::now();
        std::thread::sleep(Duration::from_millis(10));
        mock.unlock().unwrap();
        let end = Utc::now();

        assert!(mock.was_unlocked_between(start, end));
        assert!(!mock.was_unlocked_between(end, end + chrono::Duration::seconds(1)));
    }

    #[test]
    fn mock_door_lock_tracks_sensor_state() {
        let mock = MockDoorLock::new();
        assert!(!mock.is_door_open());
        mock.set_door_open(true);
        assert!(mock.is_door_open());
    }
}

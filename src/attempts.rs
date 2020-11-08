use serde::{Deserialize};
use std::time::SystemTime;
use std::time::Duration;
use rand::{Rng};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub type Count = u8;
pub type Code = u16;
pub type Phone = u64;

#[derive(Deserialize)]
pub struct Attempt {
    pub count: Count,
    pub last_attemped_at: SystemTime,
    pub code: Code
}

impl Default for Attempt {
    fn default() -> Attempt {
        Attempt {
            count: 3,
            last_attemped_at: SystemTime::now(),
            code: rand::thread_rng().gen_range(1, 9999)
        }
    }
}

pub type Attempts = HashMap<Phone, Attempt>;

pub trait PhonesNCodes {
    fn check_code(&mut self, phone: Phone, code: Code) -> CodeResult;
    fn get_code(&mut self, phone: Phone) -> PhoneResult;
}

impl PhonesNCodes for Attempts {
    fn check_code(&mut self, phone: Phone, code: Code) -> CodeResult {
        match check_phone(phone) {
            false => CodeResult::InvalidPhone,
            true => match self.entry(phone) {
                Entry::Vacant(_) => CodeResult::NotFound,
                Entry::Occupied(mut entry) => {
                    let attempt = entry.get_mut();
                    match attempt {
                        attempt if attempt.last_attemped_at.elapsed().unwrap() > Duration::from_secs(10) =>
                            CodeResult::Expired,
                        attempt if attempt.count == 0 =>
                            CodeResult::OutOfAttempts(attempt.last_attemped_at + Duration::from_secs(10)),
                        attempt if attempt.code == code => {
                            entry.remove_entry();
                            CodeResult::Valid
                        },
                        _ => {
                            attempt.count = attempt.count - 1;
                            attempt.last_attemped_at = SystemTime::now();
                            match attempt.count {
                                count if count > 0 => CodeResult::Invalid(count),
                                _ => CodeResult::OutOfAttempts(attempt.last_attemped_at + Duration::from_secs(10))
                            }
                        }
                    }
                }
            }
        }
    }
    fn get_code(&mut self, phone: Phone) -> PhoneResult {
        match check_phone(phone) {
            false => PhoneResult::InvalidPhone,
            true => match self.entry(phone) {
                Entry::Vacant(entry) => {
                    let attempt = entry.insert(Attempt::default());
                    PhoneResult::Success(attempt.count)
                },
                Entry::Occupied(mut entry) => match &entry.get() {
                    attempt if attempt.last_attemped_at.elapsed().unwrap() > Duration::from_secs(10) => {
                        entry.insert(Attempt::default());
                        PhoneResult::Success(entry.get().count)
                    },
                    attempt if attempt.count == 0 =>
                        PhoneResult::TooSoon(attempt.last_attemped_at + Duration::from_secs(10)),
                    attempt =>
                        PhoneResult::Exists(attempt.count, attempt.last_attemped_at + Duration::from_secs(10))
                }
            }
        }
    }
}

pub enum CodeResult {
    Expired,
    OutOfAttempts(SystemTime),
    Invalid(Count),
    InvalidPhone,
    NotFound,
    Valid
}

pub enum PhoneResult {
    Success(Count),
    Exists(Count, SystemTime),
    TooSoon(SystemTime),
    InvalidPhone
}

fn check_phone(phone: Phone) -> bool {
    phone < 79000000000 || phone > 79999999999
}
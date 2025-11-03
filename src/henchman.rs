//! Module to define henchmen.
#![allow(dead_code)]

/// Henchman trait.
pub trait Henchman {
    fn build_secret_hq(&mut self, location: String);
    fn do_hard_things(&self);
    fn fight_enemies(&self);
}

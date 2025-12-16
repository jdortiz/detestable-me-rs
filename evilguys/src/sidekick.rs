//! Module for sideckicks and all the related functionality
#![allow(dead_code)]

#[cfg(test)]
use mockall::mock;

use crate::Gadget;

/// Type that represents a sidekick.
pub struct Sidekick<'a> {
    gadget: Box<dyn Gadget + 'a>,
}

impl<'a> Sidekick<'a> {
    fn new<G: Gadget + 'a>(gadget: G) -> Sidekick<'a> {
        Sidekick {
            gadget: Box::new(gadget),
        }
    }

    pub fn agree(&self) -> bool {
        true
    }

    pub fn get_weak_targets<G: Gadget>(&self, _gadget: &G) -> Vec<String> {
        vec![]
    }

    pub fn tell(&self, _ciphered_msg: &str) {}
}

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Sidekick<'a> {
        pub fn agree(&self) -> bool;
        pub fn get_weak_targets(&self, _gadget: &'a dyn Gadget) -> Vec<String>;
        pub fn tell(&self, _ciphered_msg: &str);
    }
}

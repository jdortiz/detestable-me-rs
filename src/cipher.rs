//! Module for sideckicks and all the related functionality
#![allow(dead_code)]

/// Type that represents a sidekick.
pub trait Cipher {
    fn transform(&self, secret: &str, key: &str) -> String;
}

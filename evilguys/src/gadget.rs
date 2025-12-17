//! Module for gadgets and all the related functionality
#![allow(dead_code)]

#[cfg(test)]
use mockall::automock;

/// Trait that represents a gadget.
#[cfg_attr(test, automock)]
pub trait Gadget: Send {
    fn do_stuff(&self);
}

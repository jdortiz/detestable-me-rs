//! Module for gadgets and all the related functionality
#![allow(dead_code)]

/// Trait that represents a gadget.
pub trait Gadget: Send {
    fn do_stuff(&self);
}

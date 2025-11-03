//! Module for supervillains and their related stuff
#![allow(unused)]
use std::time::Duration;

use rand::Rng;
use thiserror::Error;

#[cfg(not(test))]
use crate::sidekick::Sidekick;
use crate::{Cipher, Gadget, Henchman};
#[cfg(test)]
use tests::doubles::Sidekick;

/// Type that represents supervillains.
#[derive(Default)]
pub struct Supervillain<'a> {
    pub first_name: String,
    pub last_name: String,
    pub sidekick: Option<Sidekick<'a>>,
    pub shared_key: String,
}

pub trait Megaweapon {
    fn shoot(&self);
}

impl Supervillain<'_> {
    /// Return the value of the full name as a single string.
    ///
    /// Full name is produced concatenating first name, a single space, and the last name.
    ///
    /// # Examples
    /// ```
    ///# use evil::supervillain::Supervillain;
    /// let lex = Supervillain {
    ///     first_name: "Lex".to_string(),
    ///     last_name: "Luthor".to_string(),
    /// };
    /// assert_eq!(lex.full_name(), "Lex Luthor");
    /// ```
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn set_full_name(&mut self, name: &str) {
        let components = name.split(" ").collect::<Vec<_>>();
        println!("Received {} components.", components.len());
        if components.len() != 2 {
            panic!("Name must have first and last name");
        }
        self.first_name = components[0].to_string();
        self.last_name = components[1].to_string();
    }

    pub fn attack(&self, weapon: &impl Megaweapon, intense: bool) {
        weapon.shoot();
        if intense {
            let mut rng = rand::rng();
            let times = rng.random_range(1..3);
            for _ in 0..times {
                weapon.shoot();
            }
        }
    }

    pub async fn come_up_with_plan(&self) -> String {
        tokio::time::sleep(Duration::from_millis(100)).await;
        String::from("Take over the world!")
    }

    pub fn conspire(&mut self) {
        if let Some(ref sidekick) = self.sidekick {
            if !sidekick.agree() {
                self.sidekick = None;
            }
        }
    }

    pub fn start_world_domination_stage1<H: Henchman, G: Gadget>(
        &self,
        henchman: &mut H,
        gadget: &G,
    ) {
        if let Some(ref sidekick) = self.sidekick {
            let targets = sidekick.get_weak_targets(gadget);
            if !targets.is_empty() {
                henchman.build_secret_hq(targets[0].clone());
            }
        }
    }

    pub fn start_world_domination_stage2<H: Henchman>(&self, henchman: H) {
        henchman.fight_enemies();
        henchman.do_hard_things();
    }

    pub fn tell_plans<C: Cipher>(&self, secret: &str, cipher: &C) {
        if let Some(ref sidekick) = self.sidekick {
            let ciphered_msg = cipher.transform(secret, &self.shared_key);
            sidekick.tell(&ciphered_msg);
        }
    }
}

impl TryFrom<&str> for Supervillain<'_> {
    type Error = EvilError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        let components = name.split(" ").collect::<Vec<_>>();
        if components.len() < 2 {
            Err(EvilError::ParseError {
                purpose: "full_name".to_string(),
                reason: "Too few arguments".to_string(),
            })
        } else {
            Ok(Supervillain {
                first_name: components[0].to_string(),
                last_name: components[1].to_string(),
                ..Default::default()
            })
        }
    }
}

#[derive(Error, Debug)]
pub enum EvilError {
    #[error("Parse error: purpose='{}', reason='{}'", .purpose, .reason)]
    ParseError { purpose: String, reason: String },
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use test_context::{AsyncTestContext, TestContext, test_context};

    use crate::test_common;

    use super::*;

    #[test_context(Context)]
    #[test]
    fn full_name_is_first_name_space_last_name(ctx: &mut Context) {
        let full_name = ctx.sut.full_name();

        assert_eq!(
            full_name,
            test_common::PRIMARY_FULL_NAME,
            "Unexpected full name"
        );
    }

    #[test_context(Context)]
    #[test]
    fn set_full_name_sets_first_and_last_names(ctx: &mut Context) {
        ctx.sut.set_full_name(test_common::SECONDARY_FULL_NAME);

        assert_eq!(ctx.sut.first_name, test_common::SECONDARY_FIRST_NAME);
        assert_eq!(ctx.sut.last_name, test_common::SECONDARY_LAST_NAME);
    }

    #[test_context(Context)]
    #[test]
    #[should_panic(expected = "Name must have first and last name")]
    fn set_full_name_panics_with_empty_name(ctx: &mut Context) {
        ctx.sut.set_full_name("");
    }

    #[test]
    fn try_from_str_slice_produces_supervillain_full_with_first_and_last_name()
    -> Result<(), EvilError> {
        let sut = Supervillain::try_from(test_common::SECONDARY_FULL_NAME)?;
        assert_eq!(sut.first_name, test_common::SECONDARY_FIRST_NAME);
        assert_eq!(sut.last_name, test_common::SECONDARY_LAST_NAME);
        Ok(())
    }

    #[test]
    fn try_from_str_slice_produces_error_with_less_than_two_substrings() {
        let result = Supervillain::try_from("");
        let Err(error) = result else {
            panic!("Unexpected value returned by try_from");
        };
        assert!(
            matches!(error, EvilError::ParseError { purpose, reason } if purpose =="full_name" && reason == "Too few arguments")
        )
    }

    #[test_context(Context)]
    #[test]
    fn non_intensive_attack_shoots_weapon_once(ctx: &mut Context) {
        let weapon = WeaponDouble::new();

        ctx.sut.attack(&weapon, false);

        weapon.verify(once());
    }

    #[test_context(Context)]
    #[test]
    fn intensive_attack_shoots_weapon_twice_or_more(ctx: &mut Context) {
        let weapon = WeaponDouble::new();

        ctx.sut.attack(&weapon, true);

        weapon.verify(at_least(2));
    }

    #[test_context(Context)]
    #[tokio::test]
    async fn plan_is_sadly_expected(ctx: &mut Context<'_>) {
        assert_eq!(ctx.sut.come_up_with_plan().await, "Take over the world!");
    }

    #[test_context(Context)]
    #[test]
    fn keep_sidekick_if_agrees_with_conspiracy(ctx: &mut Context) {
        let mut sk_double = doubles::Sidekick::new();
        sk_double.agree_answer = true;
        ctx.sut.sidekick = Some(sk_double);

        ctx.sut.conspire();

        assert!(ctx.sut.sidekick.is_some(), "Sidekick fired unexpectedly");
    }

    #[test_context(Context)]
    #[test]
    fn fire_sidekick_if_doesnt_agree_with_conspiracy(ctx: &mut Context) {
        let mut sk_double = doubles::Sidekick::new();
        sk_double.agree_answer = false;
        ctx.sut.sidekick = Some(sk_double);
        ctx.sut.conspire();
        assert!(
            ctx.sut.sidekick.is_none(),
            "Sidekick not fired unexpectedly"
        );
    }

    #[test_context(Context)]
    #[test]
    fn conspiracy_without_sidekick_doesnt_fail(ctx: &mut Context) {
        ctx.sut.conspire();

        assert!(ctx.sut.sidekick.is_none(), "Unexpected sidekick");
    }

    #[test_context(Context)]
    #[test]
    fn world_domination_stage1_builds_hq_in_first_weak_target(ctx: &mut Context) {
        let gdummy = GadgetDummy {};
        let mut hm_spy = HenchmanDouble::default();
        let mut sk_double = doubles::Sidekick::new();
        sk_double.targets = test_common::TARGETS.map(String::from).to_vec();
        ctx.sut.sidekick = Some(sk_double);

        ctx.sut.start_world_domination_stage1(&mut hm_spy, &gdummy);

        assert_eq!(
            hm_spy.hq_location,
            Some(test_common::FIRST_TARGET.to_string())
        );
    }

    #[test_context(Context)]
    #[test]
    fn world_domination_stage2_tells_henchman_to_do_hard_things_and_fight_with_enemies(
        ctx: &mut Context,
    ) {
        let mut henchman = HenchmanDouble::default();
        henchman.assertions = vec![Box::new(move |h| h.verify_two_things_done())];

        ctx.sut.start_world_domination_stage2(henchman);
    }

    #[test_context(Context)]
    #[test]

    fn tell_plans_sends_ciphered_message(ctx: &mut Context) {
        let mut sk_double = doubles::Sidekick::new();
        sk_double.assertions = vec![Box::new(move |s| {
            s.verify_received_msg(test_common::MAIN_CIPHERED_MESSAGE)
        })];
        ctx.sut.sidekick = Some(sk_double);
        let fake_cipher = CipherDouble {};

        ctx.sut
            .tell_plans(test_common::MAIN_SECRET_MESSAGE, &fake_cipher);
    }

    pub(crate) mod doubles {
        use std::{cell::RefCell, marker::PhantomData};

        use crate::Gadget;

        pub struct Sidekick<'a> {
            phantom: PhantomData<&'a ()>,
            pub agree_answer: bool,
            pub targets: Vec<String>,
            pub received_msg: RefCell<String>,
            pub assertions: Vec<Box<dyn Fn(&Sidekick) -> () + Send>>,
        }

        impl<'a> Sidekick<'a> {
            pub fn new() -> Sidekick<'a> {
                Sidekick {
                    phantom: PhantomData,
                    agree_answer: false,
                    targets: vec![],
                    received_msg: RefCell::new(String::from("")),
                    assertions: vec![],
                }
            }

            pub fn agree(&self) -> bool {
                self.agree_answer
            }

            pub fn get_weak_targets<G: Gadget>(&self, _gadget: &G) -> Vec<String> {
                self.targets.clone()
            }

            pub fn tell(&self, ciphered_msg: &str) {
                *self.received_msg.borrow_mut() = ciphered_msg.to_owned();
            }

            pub fn verify_received_msg(&self, expected_msg: &str) {
                assert_eq!(*self.received_msg.borrow(), expected_msg);
            }
        }

        impl Drop for Sidekick<'_> {
            fn drop(&mut self) {
                for a in &self.assertions {
                    a(self);
                }
            }
        }
    }

    struct CipherDouble;

    impl Cipher for CipherDouble {
        fn transform(&self, secret: &str, _key: &str) -> String {
            String::from("+") + secret + "+"
        }
    }

    struct GadgetDummy;

    impl Gadget for GadgetDummy {
        fn do_stuff(&self) {}
    }

    #[derive(Default)]
    struct HenchmanDouble {
        hq_location: Option<String>,
        current_invocation: Cell<u32>,
        done_hard_things: Cell<u32>,
        fought_enemies: Cell<u32>,
        assertions: Vec<Box<dyn Fn(&HenchmanDouble) -> () + Send>>,
    }

    impl HenchmanDouble {
        fn verify_two_things_done(&self) {
            assert!(self.done_hard_things.get() == 2 && self.fought_enemies.get() == 1);
        }
    }

    impl Henchman for HenchmanDouble {
        fn build_secret_hq(&mut self, location: String) {
            self.hq_location = Some(location);
        }

        fn do_hard_things(&self) {
            self.current_invocation
                .set(self.current_invocation.get() + 1);
            self.done_hard_things.set(self.current_invocation.get());
        }

        fn fight_enemies(&self) {
            self.current_invocation
                .set(self.current_invocation.get() + 1);
            self.fought_enemies.set(self.current_invocation.get());
        }
    }

    impl Drop for HenchmanDouble {
        fn drop(&mut self) {
            for a in &self.assertions {
                a(self);
            }
        }
    }

    struct WeaponDouble {
        pub times_shot: Cell<u32>,
    }
    impl WeaponDouble {
        fn new() -> WeaponDouble {
            WeaponDouble {
                times_shot: Cell::default(),
            }
        }

        fn verify<T: Fn(u32) -> bool>(&self, check: T) {
            assert!(check(self.times_shot.get()));
        }
    }
    impl Megaweapon for WeaponDouble {
        fn shoot(&self) {
            self.times_shot.set(self.times_shot.get() + 1);
        }
    }

    struct Context<'a> {
        sut: Supervillain<'a>,
    }

    impl<'a> AsyncTestContext for Context<'a> {
        async fn setup() -> Context<'a> {
            Context {
                sut: Supervillain {
                    first_name: test_common::PRIMARY_FIRST_NAME.to_string(),
                    last_name: test_common::PRIMARY_LAST_NAME.to_string(),
                    ..Default::default()
                },
            }
        }

        async fn teardown(self) {}
    }

    fn at_least(min_times: u32) -> impl Fn(u32) -> bool {
        return (move |times: u32| (times >= min_times));
    }

    fn once() -> impl Fn(u32) -> bool {
        return (move |times: u32| (times == 1));
    }
}

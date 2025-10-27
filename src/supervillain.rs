//! Module for supervillains and their related stuff
#![allow(unused)]
use std::time::Duration;

use thiserror::Error;

#[cfg(not(test))]
use crate::sidekick::Sidekick;
use crate::{Gadget, Henchman};
#[cfg(test)]
use tests::doubles::Sidekick;

/// Type that represents supervillains.
#[derive(Default)]
pub struct Supervillain<'a> {
    pub first_name: String,
    pub last_name: String,
    pub sidekick: Option<Sidekick<'a>>,
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

    pub fn attack(&self, weapon: &impl Megaweapon) {
        weapon.shoot();
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
        henchman.do_hard_things();
        henchman.fight_enemies();
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
                sidekick: None,
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
    fn attack_shoots_weapon(ctx: &mut Context) {
        let weapon = WeaponDouble::new();

        ctx.sut.attack(&weapon);

        assert!(weapon.is_shot.get());
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

    pub(crate) mod doubles {
        use std::marker::PhantomData;

        use crate::Gadget;

        pub struct Sidekick<'a> {
            phantom: PhantomData<&'a ()>,
            pub agree_answer: bool,
            pub targets: Vec<String>,
        }

        impl<'a> Sidekick<'a> {
            pub fn new() -> Sidekick<'a> {
                Sidekick {
                    phantom: PhantomData,
                    agree_answer: false,
                    targets: vec![],
                }
            }

            pub fn agree(&self) -> bool {
                self.agree_answer
            }

            pub fn get_weak_targets<G: Gadget>(&self, _gadget: &G) -> Vec<String> {
                self.targets.clone()
            }
        }
    }

    struct GadgetDummy;

    impl Gadget for GadgetDummy {
        fn do_stuff(&self) {}
    }

    #[derive(Default)]
    struct HenchmanDouble {
        hq_location: Option<String>,
        done_hard_things: Cell<bool>,
        fought_enemies: Cell<bool>,
        assertions: Vec<Box<dyn Fn(&HenchmanDouble) -> () + Send>>,
    }

    impl HenchmanDouble {
        fn verify_two_things_done(&self) {
            assert!(self.done_hard_things.get() && self.fought_enemies.get());
        }
    }

    impl Henchman for HenchmanDouble {
        fn build_secret_hq(&mut self, location: String) {
            self.hq_location = Some(location);
        }

        fn do_hard_things(&self) {
            self.done_hard_things.set(true);
        }

        fn fight_enemies(&self) {
            self.fought_enemies.set(true);
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
        pub is_shot: Cell<bool>,
    }
    impl WeaponDouble {
        fn new() -> WeaponDouble {
            WeaponDouble {
                is_shot: Cell::new(false),
            }
        }
    }
    impl Megaweapon for WeaponDouble {
        fn shoot(&self) {
            self.is_shot.set(true);
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
}

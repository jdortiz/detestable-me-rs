//! Module for supervillains and their related stuff
#![allow(unused)]
#[cfg(not(test))]
use std::fs::File;
use std::{
    io::{BufRead, BufReader, Read},
    time::Duration,
};
#[cfg(test)]
use tests::doubles::File;

#[cfg(test)]
use mockall::automock;
#[cfg(test)]
use mockall_double::double;
use rand::Rng;
use thiserror::Error;

#[cfg_attr(test, double)]
use crate::sidekick::Sidekick;
use crate::{Cipher, Gadget, Henchman};
#[cfg(not(test))]
use aux::open_buf_read;
#[cfg(test)]
use tests::doubles::open_buf_read;

const LISTING_PATH: &str = "tmp/listings.csv";

/// Type that represents supervillains.
#[derive(Default)]
pub struct Supervillain<'a> {
    pub first_name: String,
    pub last_name: String,
    pub sidekick: Option<Sidekick<'a>>,
    pub shared_key: String,
}

#[cfg_attr(test, automock)]
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

    pub fn are_there_vulnerable_locations(&self) -> Option<bool> {
        let mut listing = String::new();
        let Ok(mut file_listing) = File::open(LISTING_PATH) else {
            return None;
        };
        let Ok(n) = file_listing.read_to_string(&mut listing) else {
            return None;
        };

        for line in listing.lines() {
            if line.ends_with("weak") {
                return Some(true);
            }
        }
        Some(false)
    }

    pub fn are_there_vulnerable_locations_efficient(&self) -> Option<bool> {
        let Some(buf_listing) = open_buf_read(LISTING_PATH) else {
            return None;
        };
        let mut list_iter = buf_listing.lines();
        while let Some(line) = list_iter.next() {
            if let Ok(line) = line
                && line.ends_with("weak")
            {
                return Some(true);
            }
        }
        Some(false)
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

mod aux {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    pub fn open_buf_read(path: &str) -> Option<impl BufRead> {
        let Ok(mut file) = File::open(path) else {
            return None;
        };
        Some(BufReader::new(file))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use assertables::{assert_matches, assert_none, assert_some, assert_some_eq_x};
    use mockall::{Sequence, predicate::eq};
    use test_context::{AsyncTestContext, TestContext, test_context};

    use crate::{cipher::MockCipher, gadget::MockGadget, henchman::MockHenchman, test_common};

    use super::*;

    thread_local! {
        static FILE_IF_CAN_OPEN: RefCell<Option<doubles::File>> = RefCell::new(None);
        static FILE_CAN_OPEN: Cell<bool> = Cell::new(false);
        static BUF_CONTENTS: RefCell<String> = RefCell::new(String::new());
    }

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

        // assert2::check!(ctx.sut.first_name == "A");
        // assert2::assert!(ctx.sut.last_name == "B");
        assert2::check!(ctx.sut.first_name == test_common::SECONDARY_FIRST_NAME);
        assert2::assert!(ctx.sut.last_name == test_common::SECONDARY_LAST_NAME);
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
        assert_matches!(error, EvilError::ParseError { purpose, reason } if purpose =="full_name" && reason == "Too few arguments");
    }

    #[test_context(Context)]
    #[test]
    fn non_intensive_attack_shoots_weapon_once(ctx: &mut Context) {
        let mut weapon = MockMegaweapon::new();
        weapon.expect_shoot().once().return_const(());

        ctx.sut.attack(&weapon, false);
    }

    #[test_context(Context)]
    #[test]
    fn intensive_attack_shoots_weapon_twice_or_more(ctx: &mut Context) {
        let mut weapon = MockMegaweapon::new();
        weapon.expect_shoot().times(2..=3).return_const(());

        ctx.sut.attack(&weapon, true);
    }

    #[test_context(Context)]
    #[tokio::test]
    async fn plan_is_sadly_expected(ctx: &mut Context<'_>) {
        assert_eq!(ctx.sut.come_up_with_plan().await, "Take over the world!");
    }

    #[test_context(Context)]
    #[test]
    fn keep_sidekick_if_agrees_with_conspiracy(ctx: &mut Context) {
        let mut mock_sidekick = Sidekick::new();
        mock_sidekick.expect_agree().once().return_const(true);
        ctx.sut.sidekick = Some(mock_sidekick);

        ctx.sut.conspire();

        assert_some!(&ctx.sut.sidekick, "Sidekick fired unexpectedly");
    }

    #[test_context(Context)]
    #[test]
    fn fire_sidekick_if_doesnt_agree_with_conspiracy(ctx: &mut Context) {
        let mut mock_sidekick = Sidekick::new();
        mock_sidekick.expect_agree().once().return_const(false);
        ctx.sut.sidekick = Some(mock_sidekick);

        ctx.sut.conspire();

        assert_none!(&ctx.sut.sidekick, "Sidekick not fired unexpectedly");
    }

    #[test_context(Context)]
    #[test]
    fn conspiracy_without_sidekick_doesnt_fail(ctx: &mut Context) {
        ctx.sut.conspire();

        assert_none!(&ctx.sut.sidekick, "Unexpected sidekick");
    }

    #[test_context(Context)]
    #[test]
    fn world_domination_stage1_builds_hq_in_first_weak_target(ctx: &mut Context) {
        let gdummy = MockGadget::new();
        let mut mock_henchman = MockHenchman::new();
        mock_henchman
            .expect_build_secret_hq()
            .with(eq(String::from(test_common::FIRST_TARGET)))
            .return_const(());
        let mut mock_sidekick = Sidekick::new();
        mock_sidekick
            .expect_get_weak_targets()
            .once()
            .returning(|_| test_common::TARGETS.map(String::from).to_vec());
        ctx.sut.sidekick = Some(mock_sidekick);

        ctx.sut
            .start_world_domination_stage1(&mut mock_henchman, &gdummy);
    }

    #[test_context(Context)]
    #[test]
    fn world_domination_stage2_tells_henchman_to_do_hard_things_and_fight_with_enemies(
        ctx: &mut Context,
    ) {
        let mut mock_henchman = MockHenchman::new();
        let mut sequence = Sequence::new();
        mock_henchman
            .expect_fight_enemies()
            .once()
            .in_sequence(&mut sequence)
            .return_const(());
        mock_henchman
            .expect_do_hard_things()
            .once()
            .in_sequence(&mut sequence)
            .return_const(());

        ctx.sut.start_world_domination_stage2(mock_henchman);
    }

    #[test_context(Context)]
    #[test]
    fn tell_plans_sends_ciphered_message(ctx: &mut Context) {
        let mut mock_sidekick = Sidekick::new();
        mock_sidekick
            .expect_tell()
            .with(eq(String::from(test_common::MAIN_CIPHERED_MESSAGE)))
            .once()
            .return_const(());
        ctx.sut.sidekick = Some(mock_sidekick);
        let mut mock_cipher = MockCipher::new();
        mock_cipher
            .expect_transform()
            .returning(|secret, _| String::from("+") + secret + "+");

        ctx.sut
            .tell_plans(test_common::MAIN_SECRET_MESSAGE, &mock_cipher);
    }

    #[test_context(Context)]
    #[test]
    fn vulnerable_locations_with_no_file_returns_none(ctx: &mut Context) {
        FILE_IF_CAN_OPEN.replace(None);
        assert_none!(ctx.sut.are_there_vulnerable_locations());
    }

    #[test_context(Context)]
    #[test]
    fn vulnerable_locations_with_file_reading_error_returns_none(ctx: &mut Context) {
        FILE_IF_CAN_OPEN.replace(Some(doubles::File::new(None)));
        assert_none!(ctx.sut.are_there_vulnerable_locations());
    }

    #[test_context(Context)]
    #[test]
    fn vulnerable_locations_with_weak_returns_true(ctx: &mut Context) {
        FILE_IF_CAN_OPEN.replace(Some(doubles::File::new(Some(String::from(
            r#"Madrid,strong
               Las Vegas,weak
               New York,strong"#,
        )))));
        assert_some_eq_x!(ctx.sut.are_there_vulnerable_locations(), true);
    }

    #[test_context(Context)]
    #[test]
    fn vulnerable_locations_without_weak_returns_false(ctx: &mut Context) {
        FILE_IF_CAN_OPEN.replace(Some(doubles::File::new(Some(String::from(
            r#"Madrid,strong
               Oregon,strong
               New York,strong"#,
        )))));
        assert_some_eq_x!(ctx.sut.are_there_vulnerable_locations(), false);
    }

    #[test_context(Context)]
    #[test]
    fn efficient_vulnerable_locations_with_no_file_returns_none(ctx: &mut Context) {
        FILE_CAN_OPEN.set(false);
        assert_none!(ctx.sut.are_there_vulnerable_locations_efficient());
    }

    #[test_context(Context)]
    #[test]
    fn efficient_vulnerable_locations_with_weak_returns_true(ctx: &mut Context) {
        FILE_CAN_OPEN.set(true);
        BUF_CONTENTS.replace(String::from(
            r#"Madrid,strong
             Las Vegas,weak
             New York,strong"#,
        ));
        assert_some_eq_x!(ctx.sut.are_there_vulnerable_locations_efficient(), true);
    }

    #[test_context(Context)]
    #[test]
    fn efficient_vulnerable_locations_without_weak_returns_false(ctx: &mut Context) {
        FILE_CAN_OPEN.set(true);
        BUF_CONTENTS.replace(String::from(
            r#"Madrid,strong
             Oregon,strong
             New York,strong"#,
        ));
        assert_some_eq_x!(ctx.sut.are_there_vulnerable_locations_efficient(), false);
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

    pub mod doubles {
        use std::{
            io::{self, Cursor, Error, ErrorKind},
            path::Path,
        };

        use super::*;

        pub struct File {
            read_result: Option<String>,
        }

        impl File {
            pub fn new(read_result: Option<String>) -> File {
                File { read_result }
            }

            pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
                if let Some(file) = FILE_IF_CAN_OPEN.take() {
                    Ok(file)
                } else {
                    Err(Error::from(ErrorKind::NotFound))
                }
            }
            pub fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
                if let Some(ref content) = self.read_result {
                    *buf = content.to_owned();
                    Ok(buf.len())
                } else {
                    Err(Error::from(ErrorKind::Other))
                }
            }
        }

        pub fn open_buf_read(path: &str) -> Option<impl BufRead> {
            if FILE_CAN_OPEN.get() {
                Some(Cursor::new(BUF_CONTENTS.take()))
            } else {
                None
            }
        }
    }
}

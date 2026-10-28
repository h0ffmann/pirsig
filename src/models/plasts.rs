//! Plasts and their schedule. The exercises themselves live in
//! [`crate::plasticity`]; this keeps each plast's level, streak and due time,
//! and every attempt.
//!
//! The schedule is a small spaced repetition: a right answer pushes the plast
//! out (1 h, 2 h, 4 h, … up to 64 h) and every third one in a row raises the
//! level; a wrong answer brings it back now, and two in a row lower the level.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::plasts::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel, QueryOrder};

use super::{_entities::plast_attempts, rules};
use crate::plasticity::{self, Exercise, CATALOG, DOMAINS, MAX_LEVEL};
pub type Plasts = Entity;

/// Seeds stay below 2^53 so they survive a trip through JSON in JavaScript.
pub const MAX_SEED: u64 = (1 << 53) - 1;

macro_rules! touch_updated_at {
    ($am:ty) => {
        #[async_trait::async_trait]
        impl ActiveModelBehavior for $am {
            async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
            where
                C: ConnectionTrait,
            {
                if !insert && self.updated_at.is_unchanged() {
                    let mut this = self;
                    this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
                    Ok(this)
                } else {
                    Ok(self)
                }
            }
        }
    };
}
touch_updated_at!(ActiveModel);
touch_updated_at!(plast_attempts::ActiveModel);

/// Put every catalog plast in the brain, once.
pub async fn ensure_catalog<C: ConnectionTrait>(db: &C) -> ModelResult<()> {
    for p in &CATALOG {
        let exists = Entity::find()
            .filter(Column::Slug.eq(p.slug))
            .one(db)
            .await?
            .is_some();
        if !exists {
            ActiveModel {
                slug: Set(p.slug.to_string()),
                domain: Set(p.domain.to_string()),
                title: Set(p.title.to_string()),
                description: Set(Some(p.description.to_string())),
                // `ztk`, the app's old name, marks the shipped plasts in existing brains
                origin: Set("ztk".into()),
                level: Set(1),
                streak: Set(0),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
    }
    Ok(())
}

/// A fresh seed for the next exercise.
#[must_use]
pub fn fresh_seed() -> u64 {
    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
    u64::try_from(nanos).unwrap_or_default() & MAX_SEED
}

impl Model {
    pub async fn find<C: ConnectionTrait>(db: &C, slug: &str) -> ModelResult<Self> {
        ensure_catalog(db).await?;
        Entity::find()
            .filter(Column::Slug.eq(slug))
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    fn catalog(&self) -> ModelResult<&'static plasticity::Plast> {
        plasticity::plast(&self.slug).ok_or_else(|| {
            rules::invalid(
                "plast",
                "only the plasts Pirsig ships can be practised for now",
            )
        })
    }

    /// The exercise for `seed` at this plast's level (or `level`, when given).
    pub fn exercise(&self, seed: u64, level: Option<u8>) -> ModelResult<Exercise> {
        if seed > MAX_SEED {
            return Err(rules::invalid("seed", "seed must be below 2^53"));
        }
        let level = level.unwrap_or_else(|| u8::try_from(self.level).unwrap_or(1));
        if !(1..=MAX_LEVEL).contains(&level) {
            return Err(rules::invalid("level", "level runs from 1 to 5"));
        }
        Ok((self.catalog()?.generate)(seed, level))
    }

    /// Check `given` against the exercise `(seed, level)`, record the attempt,
    /// and move the schedule.
    pub async fn answer<C: ConnectionTrait>(
        self,
        db: &C,
        seed: u64,
        level: u8,
        given: &str,
        millis: Option<i64>,
    ) -> ModelResult<(Self, Exercise, bool)> {
        let ex = self.exercise(seed, Some(level))?;
        let correct = ex.check(given);
        plast_attempts::ActiveModel {
            plast_id: Set(self.id),
            seed: Set(i64::try_from(seed).unwrap_or_default()),
            level: Set(i32::from(level)),
            prompt: Set(ex.prompt.clone()),
            expected: Set(ex.expected()),
            answer: Set(given.trim().to_string()),
            correct: Set(correct),
            millis: Set(millis.filter(|m| *m >= 0)),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let now = chrono::Utc::now();
        let (streak, level, due) = if correct {
            let streak = self.streak + 1;
            let level = if streak % 3 == 0 {
                (self.level + 1).min(i32::from(MAX_LEVEL))
            } else {
                self.level
            };
            let hours = 1_i64 << (streak - 1).clamp(0, 6);
            (streak, level, now + chrono::Duration::hours(hours))
        } else {
            // one slip keeps the level; a second wrong answer in a row lowers it
            let level = if self.streak == 0 && self.practiced_at.is_some() {
                (self.level - 1).max(1)
            } else {
                self.level
            };
            (0, level, now)
        };
        let mut item = self.into_active_model();
        item.streak = Set(streak);
        item.level = Set(level);
        item.due_at = Set(Some(due.into()));
        item.practiced_at = Set(Some(now.into()));
        Ok((item.update(db).await?, ex, correct))
    }
}

impl Entity {
    /// The plast to practise next: never practised first, then the most overdue.
    pub async fn next<C: ConnectionTrait>(db: &C, domain: Option<&str>) -> ModelResult<Model> {
        if let Some(d) = domain {
            rules::one_of("domain", d, &DOMAINS)?;
        }
        let mut all = Self::listing(domain, db).await?;
        all.sort_by_key(|p| (p.due_at.is_some(), p.due_at, p.id));
        all.into_iter().next().ok_or(ModelError::EntityNotFound)
    }

    pub async fn listing<C: ConnectionTrait>(
        domain: Option<&str>,
        db: &C,
    ) -> ModelResult<Vec<Model>> {
        ensure_catalog(db).await?;
        let mut q = Self::find().order_by_asc(Column::Id);
        if let Some(d) = domain {
            q = q.filter(Column::Domain.eq(d));
        }
        Ok(q.all(db).await?)
    }
}

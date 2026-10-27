use std::{
    io::{BufRead, Write as _},
    time::Instant,
};

use clap::Subcommand;
use loco_rs::{app::AppContext, Error, Result};

use super::{explain, Output};
use crate::{
    dtos::plasticity::PlastDto,
    models::plasts::{self, Entity},
};

/// `ztk plast` (list), `ztk plast go -n 5`, `ztk plast go -d music`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// The plasts, with level, streak and when each is due (the default).
    List {
        /// math, abstraction, logic, music, literature or languages.
        #[arg(short, long)]
        domain: Option<String>,
    },
    /// A quick session: the most due exercises, one at a time, answered here.
    Go {
        /// Only this domain.
        #[arg(short, long)]
        domain: Option<String>,
        /// How many exercises.
        #[arg(short = 'n', long, default_value_t = 5)]
        count: u32,
    },
}

fn line(p: &PlastDto) -> String {
    let due = match p.due_at {
        None => "new".to_string(),
        Some(d) if d <= chrono::Utc::now() => "due now".to_string(),
        Some(d) => format!("due {}", d.format("%Y-%m-%d %H:%M")),
    };
    format!(
        "{:<18} {:<11} level {} · streak {:<2} · {due}",
        p.slug, p.domain, p.level, p.streak
    )
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    match args.action.unwrap_or(Action::List { domain: None }) {
        Action::List { domain } => {
            let all: Vec<PlastDto> = Entity::listing(domain.as_deref(), db)
                .await
                .map_err(explain)?
                .into_iter()
                .map(Into::into)
                .collect();
            out.many(&all, line);
        }
        Action::Go { domain, count } => {
            let mut right = 0;
            for i in 1..=count {
                let plast = Entity::next(db, domain.as_deref()).await.map_err(explain)?;
                let level = u8::try_from(plast.level).unwrap_or(1);
                let ex = plast
                    .exercise(plasts::fresh_seed(), None)
                    .map_err(explain)?;
                println!(
                    "\n[{i}/{count}] {} · level {level}\n{}",
                    plast.title, ex.prompt
                );
                for (k, c) in ex.choices.iter().enumerate() {
                    if ex.choices.len() > 2 {
                        println!("  {}) {c}", char::from(b'A' + u8::try_from(k).unwrap_or(0)));
                    }
                }
                print!("> ");
                std::io::stdout().flush().ok();
                let started = Instant::now();
                let mut answer = String::new();
                if std::io::stdin()
                    .lock()
                    .read_line(&mut answer)
                    .map_err(|e| Error::Message(e.to_string()))?
                    == 0
                {
                    break; // end of input
                }
                let millis = i64::try_from(started.elapsed().as_millis()).ok();
                let (after, ex, correct) = plast
                    .answer(db, ex.seed, level, &answer, millis)
                    .await
                    .map_err(explain)?;
                if correct {
                    right += 1;
                    println!("✓ {}", ex.explain);
                } else {
                    println!("✗ {} ({})", ex.expected(), ex.explain);
                }
                if after.level != i32::from(level) {
                    println!("  {} is now level {}", after.title, after.level);
                }
            }
            println!("\n{right}/{count} right");
        }
    }
    Ok(())
}

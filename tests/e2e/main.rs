//! End-to-end tests against a real DRACOON instance - see `harness.rs` for the test model.
//!
//! Configuration: `.env` (see `.env.example`) or environment variables.
//! Without configuration, all tests are reported as ignored.
//!
//! ```text
//! cargo test --test e2e                      # all tests (parallel)
//! cargo test --test e2e -- transfers         # filter by name
//! cargo test --test e2e -- --test-threads=1  # sequential
//! E2E_CRYPTO=1 cargo test --test e2e         # include encrypted tests (replaces the user's keypair)
//! ```

mod harness;

mod groups;
mod nodes;
mod profile;
mod rooms;
mod shares;
mod transfers;
mod users;

use std::{process::ExitCode, sync::Arc};

use harness::{BoxFuture, Config, Ctx, Kind, TestFn, TestResult, TestRun};
use libtest_mimic::{Arguments, Failed, Trial};

/// registers async test functions `async fn(&Ctx) -> TestResult` as `(name, kind, fn)`
macro_rules! tests {
    ($($kind:ident $module:ident :: $name:ident),* $(,)?) => {
        vec![$({
            fn run(ctx: &Ctx) -> BoxFuture<'_, TestResult> {
                Box::pin($module::$name(ctx))
            }
            (
                concat!(stringify!($module), "::", stringify!($name)),
                Kind::$kind,
                run as TestFn,
            )
        }),*]
    };
}

fn tests() -> Vec<(&'static str, Kind, TestFn)> {
    tests![
        Plain profile::account_information,
        Plain profile::update_account_information,
        Plain nodes::create_get_update_delete,
        Plain nodes::list_children,
        Plain nodes::get_node_from_path,
        Plain nodes::search,
        Plain nodes::copy_and_move,
        Plain nodes::delete_nodes,
        Plain rooms::create_get_update_delete,
        Plain rooms::config,
        Plain rooms::room_users,
        Plain rooms::room_groups,
        Plain groups::create_get_update_delete,
        Plain groups::list_filtered_by_name,
        Plain groups::members,
        Plain groups::last_admin_rooms,
        Plain users::create_get_update_delete,
        Plain users::list_filtered_by_user_name,
        Plain users::last_admin_rooms,
        Plain shares::download_share_create_get_update_delete,
        Plain shares::download_shares_filtered_by_node,
        Plain shares::download_shares_bulk_delete,
        Plain shares::download_share_password_protected,
        Plain shares::upload_share_create_get_update_delete,
        Plain shares::upload_shares_filtered_by_target,
        Plain shares::upload_shares_bulk_delete,
        Plain transfers::small_file,
        Plain transfers::empty_file,
        Plain transfers::multi_chunk_file,
        Plain transfers::conflict_autorename,
        Plain transfers::download_range,
        Crypto transfers::encrypted_small_file,
        Crypto transfers::encrypted_empty_file,
        Crypto transfers::encrypted_multi_chunk_file,
        Crypto transfers::encrypted_download_range_is_unsupported,
    ]
}

fn main() -> ExitCode {
    let args = Arguments::from_args();
    let config = Config::from_env();
    let runtime = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
    let run = config
        .as_ref()
        .ok()
        .cloned()
        .map(|config| Arc::new(TestRun::new(config)));

    match &config {
        Err(missing) => eprintln!(
            "e2e: configuration missing ({}) - tests are ignored, see .env.example",
            missing.join(", ")
        ),
        Ok(config) if !config.crypto => {
            eprintln!("e2e: encrypted tests are ignored - enable with E2E_CRYPTO=1")
        }
        Ok(_) => {}
    }

    let trials = tests()
        .into_iter()
        .map(|(name, kind, test)| {
            let ignored = match &config {
                Err(_) => true,
                Ok(config) => kind == Kind::Crypto && !config.crypto,
            };
            let run = run.clone();
            let handle = runtime.handle().clone();

            Trial::test(name, move || {
                let Some(run) = run else {
                    return Err(Failed::from("E2E configuration missing - see .env.example"));
                };
                if kind == Kind::Crypto && !run.config.crypto {
                    return Err(Failed::from(
                        "encrypted tests disabled - enable with E2E_CRYPTO=1",
                    ));
                }
                handle
                    .block_on(run.run_test(name, kind, test))
                    .map_err(Failed::from)
            })
            .with_ignored_flag(ignored)
        })
        .collect();

    let conclusion = libtest_mimic::run(&args, trials);

    // remove the run rooms - also if tests failed
    if let Some(run) = run {
        let errors = runtime.block_on(run.cleanup());
        if !errors.is_empty() {
            for error in errors {
                eprintln!("e2e: cleanup failed - remove manually: {error}");
            }
            return ExitCode::FAILURE;
        }
    }

    conclusion.exit_code()
}

mod options {
    use argh::FromArgs;
    use std::path::PathBuf;

    #[derive(FromArgs)]
    #[argh(name = "gio-plumbing")]
    /// The lean git underworld
    pub struct Args {
        #[argh(switch)]
        /// print the program version.
        pub version: bool,

        #[argh(subcommand)]
        pub subcommand: SubCommands,
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand)]
    pub enum SubCommands {
        VerifyPack(VerifyPack),
    }

    /// Initialize the repository in the current directory.
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "verify-pack")]
    pub struct VerifyPack {
        /// output statistical information about the pack
        #[argh(switch, short = 's')]
        pub statistics: bool,
        /// verbose progress messages are printed line by line
        #[argh(switch, short = 'v')]
        pub verbose: bool,
        /// the '.pack' or '.idx' file whose checksum to validate.
        #[argh(positional)]
        pub path: PathBuf,
    }
}

use anyhow::Result;
use gitoxide_core as core;
use std::io::{stderr, stdout};

#[cfg(not(any(
    feature = "prodash-line-renderer-crossterm",
    feature = "prodash-line-renderer-termion"
)))]
fn prepare(verbose: bool, name: &str) -> ((), Option<git_features::progress::Log>) {
    super::init_env_logger(verbose);
    ((), Some(git_features::progress::Log::new(name, Some(1))))
}

#[cfg(any(
    feature = "prodash-line-renderer-crossterm",
    feature = "prodash-line-renderer-termion"
))]
fn prepare(verbose: bool, name: &str) -> (Option<prodash::line::JoinHandle>, Option<prodash::tree::Item>) {
    super::init_env_logger(false);

    if verbose {
        let progress = prodash::Tree::new();
        let sub_progress = progress.add_child(name);
        let output_is_terminal = atty::is(atty::Stream::Stderr);
        let handle = prodash::line::render(
            stderr(),
            progress,
            prodash::line::Options {
                level_filter: Some(std::ops::RangeInclusive::new(2, 2)),
                frames_per_second: crate::shared::DEFAULT_FRAME_RATE,
                initial_delay: Some(std::time::Duration::from_millis(1000)),
                output_is_terminal,
                colored: output_is_terminal && crosstermion::color::allowed(),
                timestamp: true,
                ..prodash::line::Options::default()
            },
        );
        (Some(handle), Some(sub_progress))
    } else {
        (None, None)
    }
}

pub fn main() -> Result<()> {
    pub use options::*;
    let cli: Args = crate::shared::from_env();
    match cli.subcommand {
        SubCommands::VerifyPack(VerifyPack {
            path,
            verbose,
            statistics,
        }) => {
            let (_handle, progress) = prepare(verbose, "verify-pack");
            core::verify_pack_or_pack_index(
                path,
                progress,
                if statistics {
                    Some(core::OutputFormat::Human)
                } else {
                    None
                },
                stdout(),
                stderr(),
            )
            .map(|_| ())
        }
    }
}

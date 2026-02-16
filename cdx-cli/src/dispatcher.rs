//! Command dispatch.

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;
use std::io;

use crate::cli::{Cli, Commands};
use crate::commands;
use crate::output;

#[allow(clippy::too_many_lines)] // flat match dispatching each CLI subcommand — no shared logic to extract
pub fn run_command(command: Commands, output_config: &output::OutputConfig) -> Result<()> {
    match command {
        Commands::Create {
            title,
            author,
            state,
            input,
            output: output_path,
        } => commands::create::run(&title, &author, &state, input, &output_path, output_config),

        Commands::Validate { file } => commands::validate::run(&file, output_config),

        Commands::Inspect {
            file,
            blocks,
            signatures,
            provenance,
        } => commands::inspect::run(&file, blocks, signatures, provenance, output_config),

        Commands::Status { file } => commands::status::run(&file, output_config),

        Commands::Sign {
            file,
            key,
            name,
            email,
            algorithm,
            output: output_path,
        } => commands::sign::run(
            &file,
            &key,
            &name,
            email,
            &algorithm,
            output_path,
            output_config,
        ),

        Commands::Verify { file, key } => commands::verify::run(&file, &key, output_config),

        Commands::Extract {
            file,
            output: output_path,
            content,
            text,
            asset,
            all_assets,
        } => commands::extract::run(
            &file,
            &output_path,
            content,
            text,
            asset.as_deref(),
            all_assets,
            output_config,
        ),

        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "cdx", &mut io::stdout());
            Ok(())
        }

        Commands::SubmitReview { file, output } => {
            commands::review::run(&file, output, output_config)
        }

        Commands::Freeze { file, output } => commands::freeze::run(&file, output, output_config),

        Commands::Publish { file, output } => commands::publish::run(&file, output, output_config),

        Commands::Revert { file, output } => commands::revert::run(&file, output, output_config),

        Commands::Fork { file, output, note } => {
            commands::fork::run(&file, &output, note, output_config)
        }

        Commands::Prove {
            file,
            block_id,
            block_index,
            output,
        } => commands::prove::run_prove(&file, block_id, block_index, output, output_config),

        Commands::VerifyProof { file, proof } => {
            commands::prove::run_verify_proof(&file, &proof, output_config)
        }

        Commands::ShowLineage { file } => commands::prove::run_show_lineage(&file, output_config),

        Commands::GetMetadata { file } => {
            commands::metadata::run_get_metadata(&file, output_config)
        }

        Commands::SetMetadata {
            file,
            title,
            creator,
            subject,
            description,
            publisher,
            language,
            rights,
            output,
        } => {
            let params = commands::metadata::SetMetadataParams {
                file,
                title,
                creator,
                subject,
                description,
                publisher,
                language,
                rights,
                output,
            };
            commands::metadata::run_set_metadata(&params, output_config)
        }

        Commands::Pack {
            input,
            output: output_path,
            from_json,
        } => commands::pack::run(&input, &output_path, from_json, output_config),

        Commands::Diff { file1, file2 } => commands::diff::run(&file1, &file2, output_config),

        Commands::ShowTimestamps { file } => {
            commands::timestamp::run_show_timestamps(&file, output_config)
        }

        Commands::VerifyTimestamps { file } => {
            commands::timestamp::run_verify_timestamps(&file, output_config)
        }

        Commands::AddTimestamp {
            file,
            method,
            authority,
            token,
            time,
            transaction_id,
            output,
        } => {
            let params = commands::timestamp::AddTimestampParams {
                file,
                method,
                authority,
                token,
                time,
                transaction_id,
                _output: output,
            };
            commands::timestamp::run_add_timestamp(&params, output_config)
        }

        Commands::TimestampAcquire {
            file,
            method,
            server,
            output,
        } => commands::timestamp::run_acquire_timestamp(
            &file,
            method.as_deref(),
            server.as_deref(),
            output,
            output_config,
        ),

        Commands::Encrypt {
            file,
            password,
            output,
        } => commands::encrypt::run(&file, password, output, output_config),

        Commands::Decrypt {
            file,
            password,
            output,
        } => commands::decrypt::run(&file, password, output, output_config),
    }
}

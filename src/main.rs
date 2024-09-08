use clap::Parser;
use folder_summary::{
    analyzer::analyze_code_files,
    cache::Cache,
    config::Config,
    llm::get_llm,
    summary::generate_summary,
    utils::{
        collect_code_files, collect_documentation_files, parse_package_files,
    },
};

use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn, error};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{io, path::PathBuf};
use std::path::Path;

pub type ThreadSafeCache = Arc<Mutex<Cache>>;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = ".")]
    directory: PathBuf,

    #[clap(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[clap(short, long)]
    llm_provider: Option<String>,

    #[clap(long)]
    file_types: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    // Load config file
    let mut config = Config::load(args.config.to_str().unwrap())?;

    // Override config with CLI arguments
    if let Some(llm_provider) = args.llm_provider {
        config.llm_provider = Some(llm_provider);
    }

    let llm = get_llm(&config)?;

    println!("Starting folder summary task");
    println!("Using LLM model: {}", llm.model_name());
    println!("Folder to analyze: {}", args.directory.display());

    print!("Do you want to proceed? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        warn!("User aborted the operation");
        return Ok(());
    }

    info!("Collecting files...");
    let docs = collect_documentation_files(&args.directory);
    let package_info = parse_package_files(&args.directory);
    let code_files = collect_code_files(&args.directory, &config);
    if code_files.is_empty() {
        error!("No code files found to analyze. Please check your configuration and directory path.");
        return Ok(());
    }

    let pb = ProgressBar::new(code_files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let cache = Arc::new(Mutex::new(Cache::new("analysis_cache.json")?));
    let code_analysis = analyze_code_files(&code_files, &llm, &pb, &cache).await?;

    pb.finish_with_message("Analysis complete");

    println!("Generating summary...");
    let analyzed_folder = Path::new(&args.directory);
    generate_summary(docs, package_info, code_analysis, &config, analyzed_folder);

    println!("Summary generation complete!");
    info!("Congratulations! Your folder summary is ready.");
    println!("You can find the summary at: {}", config.get_summary_output_path().display());

    Ok(())
}

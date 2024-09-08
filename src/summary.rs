use crate::analyzer::CodeAnalysis;
use crate::config::Config;
use log::info;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::utils::file_utils::get_project_name;

pub fn generate_summary(
    docs: Vec<String>,
    package_info: HashMap<String, String>,
    analysis: HashMap<String, CodeAnalysis>,
    config: &Config,
    analyzed_folder: &Path,
) {
    info!("Generating summary...");
    let mut summary = String::new();

    summary.push_str("# Code Summary\n\n");

    summary.push_str("## Documentation Files\n");
    for doc in docs {
        summary.push_str(&format!("- {}\n", doc));
    }

    summary.push_str("\n## Package Information\n");
    for (package, version) in package_info {
        summary.push_str(&format!("- {}: {}\n", package, version));
    }

    summary.push_str("\n## Code Analysis\n");
    for (file_path, code_analysis) in analysis {
        summary.push_str(&format!("## {}\n\n", file_path));

        if !code_analysis.imports.is_empty() {
            summary.push_str("**Imports:**\n");
            for import in &code_analysis.imports {
                summary.push_str(&format!("- {}\n", import));
            }
            summary.push('\n');
        }

        if !code_analysis.functions.is_empty() {
            summary.push_str("**Functions:**\n");
            for func in &code_analysis.functions {
                summary.push_str(&format!("- {}\n", func.name));
                summary.push_str(&format!("  Signature: {}\n", func.signature));
                summary.push_str(&format!("  Lines of code: {}\n", func.lines_of_code));
                summary.push_str(&format!(
                    "  Cyclomatic complexity: {}\n",
                    func.cyclomatic_complexity
                ));
                summary.push_str(&format!("  Parameters: {}\n", func.parameters));
                summary.push_str(&format!("  Returns: {}\n", func.returns));
                if let Some(sum) = &func.summary {
                    summary.push_str(&format!("  Summary: {}\n", sum));
                }
                summary.push('\n');
            }
        }

        if !code_analysis.types.is_empty() {
            summary.push_str("**Types:**\n");
            for type_def in &code_analysis.types {
                summary.push_str(&format!("```rust\n{}\n```\n\n", type_def));
            }
        }

        if !code_analysis.exports.is_empty() {
            summary.push_str("**Exports:**\n");
            for export in &code_analysis.exports {
                summary.push_str(&format!("- {}\n", export));
            }
            summary.push('\n');
        }

        summary.push_str("\n\n");
    }

    let project_name = get_project_name(analyzed_folder)
        .or_else(|| {
            analyzed_folder
                .file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "unknown".to_string());

    let output_path = config.get_summary_output_path();
    fs::create_dir_all(&output_path).expect("Unable to create summary output directory");

    let filename = config.get_summary_filename(&project_name);
    let summary_path = output_path.join(filename);

    fs::write(&summary_path, summary).expect("Unable to write summary");
    println!("Summary generated and saved as {}", summary_path.display());
}

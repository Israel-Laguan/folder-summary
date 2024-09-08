use super::{
    static_analysis::{extract_function_metrics, FunctionAnalysis},
    CodeAnalysis, LanguageAnalyzer,
};
use crate::error::FolderSummaryError;
use crate::llm::LLM;
use async_trait::async_trait;
use syn::parse_file;
use quote::ToTokens;

pub struct RustAnalyzer;

impl RustAnalyzer {
    fn extract_imports(ast: &syn::File) -> Vec<String> {
        ast.items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Use(item_use) = item {
                    Some(item_use.to_token_stream().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    fn extract_types(ast: &syn::File) -> Vec<String> {
        ast.items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Struct(item_struct) => Some(item_struct.to_token_stream().to_string()),
                syn::Item::Enum(item_enum) => Some(item_enum.to_token_stream().to_string()),
                syn::Item::Type(item_type) => Some(item_type.to_token_stream().to_string()),
                syn::Item::Impl(item_impl) => {
                    let mut item_impl = item_impl.clone();
                    item_impl.generics = item_impl.generics.clone();
                    Some(item_impl.to_token_stream().to_string())
                }
                _ => None,
            })
            .collect()
    }

    fn extract_exports(ast: &syn::File) -> Vec<String> {
        ast.items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Fn(item_fn) if matches!(item_fn.vis, syn::Visibility::Public { .. }) => {
                    Some(format!("fn {}", item_fn.sig.ident))
                }
                syn::Item::Struct(item_struct)
                    if matches!(item_struct.vis, syn::Visibility::Public(..)) =>
                {
                    Some(format!("struct {}", item_struct.ident))
                }
                syn::Item::Enum(item_enum)
                    if matches!(item_enum.vis, syn::Visibility::Public(..)) =>
                {
                    Some(format!("enum {}", item_enum.ident))
                }
                syn::Item::Type(item_type)
                    if matches!(item_type.vis, syn::Visibility::Public { .. }) =>
                {
                    Some(format!("type {}", item_type.ident))
                }
                _ => None,
            })
            .collect()
    }
}

#[async_trait]
impl LanguageAnalyzer for RustAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        file_path.ends_with(".rs")
    }

    fn analyze(&self, content: &str) -> Result<CodeAnalysis, FolderSummaryError> {
        let ast =
            parse_file(content).map_err(|e| FolderSummaryError::AnalysisError(e.to_string()))?;

        let imports = Self::extract_imports(&ast);
        let types = Self::extract_types(&ast);
        let exports = Self::extract_exports(&ast);

        let functions: Vec<FunctionAnalysis> = ast
            .items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Fn(func) = item {
                    Some(extract_function_metrics(func))
                } else {
                    None
                }
            })
            .collect();

        Ok(CodeAnalysis {
            imports,
            functions,
            types,
            exports,
        })
    }

    async fn summarize(
        &self,
        analysis: &CodeAnalysis,
        llm: &Box<dyn LLM>,
    ) -> Result<CodeAnalysis, FolderSummaryError> {
        let mut summarized = analysis.clone();
        for func in &mut summarized.functions {
            let prompt = if func.lines_of_code > 200 {
                generate_large_function_prompt(func)
            } else {
                format!(
                    "Summarize the following Rust function:\n\nName: {}\nSignature: {}\nTypes: {}\nBody: {}",
                    func.name,
                    func.signature,
                    func.types,
                    func.body.as_deref().unwrap_or("(function body omitted)")
                )
            };
        
            func.summary = Some(llm.summarize(&prompt).await?);
        }
        Ok(summarized)
    }
}

fn generate_large_function_prompt(func: &FunctionAnalysis) -> String {
    let mut prompt = format!(
        "Summarize this large Rust function:\n\nName: {}\nSignature: {}\nTypes: {}\n\nFunction body in parts:\n",
        func.name,
        func.signature,
        func.types
    );

    let body = func.body.as_deref().unwrap_or("");
    let lines: Vec<&str> = body.lines().collect();
    let chunk_size = 5;

    for (i, chunk) in lines.chunks(chunk_size).enumerate() {
        prompt.push_str(&format!("\nPart {}:\n", i + 1));
        for line in chunk {
            prompt.push_str(&format!("{}\n", line));
        }
        prompt.push_str(&format!("\nTypes: {}\n", func.types));
    }

    prompt.push_str("\nPlease provide a summary of the function's purpose and behavior based on these parts.");
    prompt
}

impl From<Box<dyn std::error::Error>> for FolderSummaryError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        FolderSummaryError::AnalysisError(err.to_string())
    }
}
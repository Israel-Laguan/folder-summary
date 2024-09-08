use proc_macro2::{LineColumn, Span};
use quote::ToTokens;
use regex::Regex;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    ExprBox, ItemFn,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionAnalysis {
    pub name: String,
    pub signature: String,
    pub types: String,
    pub body: Option<String>,
    pub lines_of_code: usize,
    pub cyclomatic_complexity: usize,
    pub parameters: usize,
    pub returns: bool,
    pub summary: Option<String>,
}

pub fn extract_function_metrics(func: &ItemFn) -> FunctionAnalysis {
    let name = func.sig.ident.to_string();
    let (signature, types) = extract_signature_and_types(&func.sig);
    let body = extract_function_body(func);
    let lines_of_code = count_lines_of_code(func);
    let cyclomatic_complexity = calculate_cyclomatic_complexity(func);
    let parameters = func.sig.inputs.len();
    let returns = func.sig.output != syn::ReturnType::Default;

    FunctionAnalysis {
        name,
        signature,
        types,
        body: if lines_of_code <= 20 { Some(body) } else { None },
        lines_of_code,
        cyclomatic_complexity,
        parameters,
        returns,
        summary: None,
    }
}

fn extract_signature_and_types(sig: &syn::Signature) -> (String, String) {
    let full_signature = sig.to_token_stream().to_string();
    let types = extract_types_from_signature(&full_signature);
    let signature = remove_body_from_signature(&full_signature);
    (signature, types)
}

fn extract_types_from_signature(signature: &str) -> String {
    let re = Regex::new(r":\s*([^,\)]+)").unwrap();
    let types: Vec<String> = re.captures_iter(signature)
        .map(|cap| cap[1].trim().to_string())
        .collect();
    types.join(", ")
}

fn remove_body_from_signature(signature: &str) -> String {
    let re = Regex::new(r"\{.*\}").unwrap();
    re.replace_all(signature, "{ ... }").to_string()
}

fn extract_function_body(func: &ItemFn) -> String {
    func.block.to_token_stream().to_string()
}

fn count_lines_of_code(func: &ItemFn) -> usize {
    let span: Span = func.span();
    let start: LineColumn = span.start();
    let end: LineColumn = span.end();
    end.line.saturating_sub(start.line) + 1
}

fn calculate_cyclomatic_complexity(func: &ItemFn) -> usize {
    let mut visitor = ComplexityVisitor { complexity: 1 };
    visitor.visit_item_fn(func);
    visitor.complexity
}

struct ComplexityVisitor {
    complexity: usize,
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.complexity += 1;
        visit::visit_expr_if(self, node);
    }

    fn visit_expr_match(&mut self, i: &'ast syn::ExprMatch) {
        self.complexity += i.arms.len();
        visit::visit_expr_match(self, i);
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.complexity += 1;
        visit::visit_expr_while(self, node);
    }

    fn visit_expr_box(&mut self, node: &'ast ExprBox) {
        self.complexity += 1;
        visit::visit_expr_box(self, node);
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.complexity += 1;
        visit::visit_expr_loop(self, node);
    }
}

# Folder Summary

This Rust project analyzes code files in a directory, extracts information about imports, functions, returns, types, and exports, and generates a summary using AI-powered function descriptions.

## Features

- Supports Rust, JavaScript/TypeScript, and Python files
- Integrates with various LLM providers (Ollama, Gemini, OpenAI)
- Respects .gitignore files
- Generates a markdown summary of the analyzed code

## Setup

1. Ensure you have Rust and Cargo installed.
2. Clone this repository.
3. Run `cargo build` to compile the project.
4. Set up your .env file with the appropriate LLM provider and API keys.

## Usage

Run the program with:

```
cargo run -- [directory_path]
```

Running without directory path will run the current folder.

The program will analyze the specified directory and generate a `summary.md` file with the results.

To run the program with logging enabled, use:

```
RUST_LOG=info cargo run -- --directory [directory_path]
```

This will show info-level log messages. You can adjust the log level (e.g., debug, warn) as needed.

If you want to format the code using the terminal, run

`cargo fmt`

## Configuration

Edit the .env file to change the LLM provider and other settings:

- LLM_PROVIDER: ollama, gemini, or openai
- OLLAMA_MODEL: The Ollama model to use
- GEMINI_API_KEY: Your Gemini API key
- GEMINI_MODEL: The Gemini model to use
- OPENAI_API_KEY: Your OpenAI API key
- OPENAI_MODEL: The OpenAI model to use
- CUSTOM_OPENAI_URL: Custom URL for OpenAI API (optional)

## Use cargo install for Local Installation

`cargo install --path .`

This will build and install the binary into `~/.cargo/bin`, which is usually part of your `PATH`, making it available as a system command.

## FAQ

### `error: failed to run custom build command for 'openssl-sys v0.x.y'`

`reqwest` have this dependency on linux, you should install openssl on your own, check here https://docs.rs/openssl or if in debian try

`sudo apt i libssl-dev`

More info: https://crates.io/crates/reqwest#requirements

## License

This project is licensed under the MIT License.

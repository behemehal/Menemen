//! Menemen CLI - A curl-like HTTP client
//!
//! Usage examples:
//! - Simple GET:     menemen https://example.com
//! - With headers:   menemen -H "Auth: token" https://example.com
//! - POST JSON:      menemen --json '{"key":"value"}' https://api.example.com
//! - Form data:      menemen --form key=value --form a=b https://api.example.com
//! - Include headers: menemen -i https://example.com
//! - Verbose output: menemen -v https://example.com
//! - Save to file:   menemen -o response.txt https://example.com

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::fs;

#[derive(Parser, Debug)]
#[command(
    name = "menemen",
    about = "A lightweight curl-like HTTP client powered by Menemen",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Default to request subcommand if no subcommand given
    #[command(flatten)]
    request: Option<RequestArgs>,
}

#[derive(Parser, Debug)]
struct RequestArgs {
    /// URL to request
    url: Option<String>,

    /// HTTP method (GET, POST, PUT, DELETE)
    #[arg(short = 'X', long, value_name = "METHOD")]
    method: Option<String>,

    /// Add a header (repeatable)
    #[arg(short = 'H', long, value_name = "HEADER")]
    header: Vec<String>,

    /// POST JSON data (auto-sets Content-Type)
    #[arg(long, value_name = "JSON", conflicts_with_all = ["form", "multipart", "data"])]
    json: Option<String>,

    /// POST raw data
    #[arg(short = 'd', long, value_name = "DATA", conflicts_with_all = ["json", "form", "multipart"])]
    data: Option<String>,

    /// Form field (repeatable, conflicts with json/multipart/data)
    #[arg(short = 'F', long, value_name = "KEY=VALUE", conflicts_with_all = ["json", "data", "multipart"])]
    form: Vec<String>,

    /// Multipart file field (repeatable, format: name=@path)
    #[arg(long, value_name = "NAME=@PATH", conflicts_with_all = ["json", "data", "form"])]
    multipart: Vec<String>,

    /// Include response headers in output
    #[arg(short = 'i', long)]
    include: bool,

    /// Verbose mode - show request and response metadata
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Write response body to file instead of stdout
    #[arg(short = 'o', long, value_name = "FILE")]
    output: Option<String>,

    /// Pretty-print JSON responses
    #[arg(long)]
    pretty_json: bool,

    /// Request timeout in milliseconds
    #[arg(long, value_name = "MS", default_value = "5000")]
    timeout: u64,

    /// Do not follow redirects
    #[arg(long)]
    no_follow: bool,

    /// Color output: auto, always, never
    #[arg(long, value_name = "WHEN", default_value = "auto")]
    color: ColorWhen,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum ColorWhen {
    /// Colorize only when stdout is a terminal and NO_COLOR is unset
    Auto,
    Always,
    Never,
}

/// Wraps text in ANSI SGR codes when coloring is enabled.
#[derive(Copy, Clone, Debug)]
struct Style {
    enabled: bool,
}

impl Style {
    fn new(when: ColorWhen) -> Self {
        let enabled = match when {
            ColorWhen::Always => true,
            ColorWhen::Never => false,
            ColorWhen::Auto => {
                use std::io::IsTerminal;
                std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
            }
        };
        Style { enabled }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.enabled {
            let esc = 27 as char;
            format!("{esc}[{code}m{text}{esc}[0m")
        } else {
            text.to_string()
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Make HTTP request (default)
    Request(RequestArgs),

    /// Generate shell completions
    Completions {
        /// Shell type: bash, zsh, fish, powershell, elvish
        #[arg(value_name = "SHELL")]
        shell: Shell,
    },

    /// Show version
    Version,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.command.is_none() && cli.request.as_ref().and_then(|r| r.url.as_ref()).is_none() {
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!();
        return Ok(());
    }

    match cli.command {
        Some(Commands::Version) => {
            println!("menemen {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(Commands::Completions { shell }) => {
            handle_completions(shell)?;
            Ok(())
        }
        Some(Commands::Request(args)) => handle_request(args),
        None => handle_request(cli.request.unwrap_or_default()),
    }
}

impl Default for RequestArgs {
    fn default() -> Self {
        Self {
            url: None,
            method: None,
            header: vec![],
            json: None,
            data: None,
            form: vec![],
            multipart: vec![],
            include: false,
            verbose: false,
            output: None,
            pretty_json: false,
            timeout: 5000,
            no_follow: false,
            color: ColorWhen::Auto,
        }
    }
}

fn handle_request(args: RequestArgs) -> Result<(), Box<dyn std::error::Error>> {
    let url = args.url.ok_or("URL is required")?;

    if args.verbose {
        eprintln!("==> REQUEST");
        eprintln!(
            "Method: {}",
            args.method.as_ref().unwrap_or(&"GET".to_string())
        );
        eprintln!("URL: {}", url);
        if !args.header.is_empty() {
            eprintln!("Headers:");
            for h in &args.header {
                eprintln!("  {}", h);
            }
        }
    }

    // Parse method
    let method = match args
        .method
        .as_deref()
        .unwrap_or("GET")
        .to_uppercase()
        .as_str()
    {
        "GET" => menemen::request::RequestTypes::GET,
        "POST" => menemen::request::RequestTypes::POST,
        "PUT" => menemen::request::RequestTypes::PUT,
        "DELETE" => menemen::request::RequestTypes::DELETE,
        m => return Err(format!("Unsupported method: {}", m).into()),
    };

    let mut request = menemen::request::Request::new(&url, method)?;

    // Set timeout
    request.set_timeout(args.timeout);
    request.set_follow_redirects(!args.no_follow);

    // Add headers
    for header in args.header {
        let parts: Vec<&str> = header.splitn(2, ':').collect();
        if parts.len() == 2 {
            request.set_header(parts[0].trim(), parts[1].trim());
        } else {
            eprintln!("Warning: Invalid header format '{}', skipping", header);
        }
    }

    // Handle body
    if let Some(json_str) = &args.json {
        request.set_header("Content-Type", "application/json");
        request.append_body(json_str.clone().into());
        if args.verbose {
            eprintln!("Body (JSON): {}", json_str);
        }
    } else if let Some(data_str) = &args.data {
        request.append_body(data_str.clone().into());
        if args.verbose {
            eprintln!("Body (raw): {}", data_str);
        }
    } else if !args.form.is_empty() {
        use menemen::form_data::FormData;
        let mut form_data = FormData::new();
        for field in &args.form {
            let parts: Vec<&str> = field.splitn(2, '=').collect();
            if parts.len() == 2 {
                form_data.add(parts[0], parts[1]);
            } else {
                eprintln!("Warning: Invalid form field '{}', skipping", field);
            }
        }
        request.set_header("Content-Type", "application/x-www-form-urlencoded");
        request.append_body(form_data.into());
        if args.verbose {
            eprintln!("Body (form): {} fields", args.form.len());
        }
    } else if !args.multipart.is_empty() {
        use menemen::prelude::MultipartFormData;

        let mut multipart = MultipartFormData::new();
        for field in &args.multipart {
            let parts: Vec<&str> = field.splitn(2, '=').collect();
            if parts.len() != 2 {
                eprintln!(
                    "Warning: Invalid multipart field '{}', expected NAME=@PATH or NAME=VALUE, skipping",
                    field
                );
                continue;
            }

            let (name, value) = (parts[0], parts[1]);
            match value.strip_prefix('@') {
                Some(path) => multipart.add_file(name, path)?,
                None => multipart.add_string(name, value.to_string()),
            }
        }

        // The client fills in Content-Type with the generated boundary.
        request.append_body(multipart.into());
        if args.verbose {
            eprintln!("Body (multipart): {} fields", args.multipart.len());
        }
    }

    // Send request
    if args.verbose {
        eprintln!("\n==> RESPONSE");
    }

    let mut response = request.send()?;

    if args.verbose {
        eprintln!("Status: {}", response.response_info.status_code);
        eprintln!("Headers: {} items", response.headers.len());
    }

    let body = response.text()?;

    // Handle output
    let output_text = if args.pretty_json && is_json_response(&response) {
        pretty_print_json(&body)?
    } else {
        body
    };

    if let Some(output_file) = args.output {
        fs::write(&output_file, &output_text)?;
        eprintln!("Wrote response to '{}'", output_file);
    } else {
        // Include headers if requested
        if args.include {
            let style = Style::new(args.color);
            let status = response.response_info.status_code;
            let status_code = match status {
                200..=299 => style.paint("32", &status.to_string()),
                300..=399 => style.paint("33", &status.to_string()),
                _ => style.paint("31", &status.to_string()),
            };

            println!(
                "{} {} {}",
                style.paint("1", &response.response_info.http_version),
                status_code,
                response.response_info.status_message
            );
            for header in &response.headers {
                println!("{}: {}", style.paint("36", &header.name), header.value);
            }
            println!();
        }
        println!("{}", output_text);
    }

    Ok(())
}

fn is_json_response(response: &menemen::response::Response) -> bool {
    response
        .headers
        .iter()
        .any(|h| h.name.to_lowercase() == "content-type" && h.value.contains("application/json"))
}

fn pretty_print_json(json_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(feature = "json")]
    {
        let parsed: serde_json::Value = serde_json::from_str(json_str)?;
        Ok(serde_json::to_string_pretty(&parsed)?)
    }
    #[cfg(not(feature = "json"))]
    {
        Ok(json_str.to_string())
    }
}

fn handle_completions(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
    Ok(())
}

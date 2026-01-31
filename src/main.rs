use clap::Parser;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const PROG: &str = "web-scraper";
const VERSION: &str = "1g.0.0";
const AUTHOR: &str = "Al Biheiri (al@forgottheaddress.com)";
const HTTP_TIMEOUT: u64 = 10;

#[derive(Parser)]
#[command(name = PROG, version = VERSION, author = AUTHOR, about = "Web Scraper")]
struct Args {
    /// URL to scrape
    url: String,

    /// Maximum depth to scrape
    #[arg(short = 'm', long, default_value_t = 1)]
    max_depth: u32,

    /// Filter results by file extension, e.g., 'mkv' (without the dot)
    #[arg(short, long)]
    filter: Option<String>,
}

struct Scraper {
    client: Client,
    visited_urls: HashSet<String>,
    filter_ext: Option<String>,
    running: Arc<AtomicBool>,
}

impl Scraper {
    fn new(filter: Option<String>, running: Arc<AtomicBool>) -> Self {
        let filter_ext = filter.map(|f| format!(".{}", f.to_lowercase()));
        let client = Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            visited_urls: HashSet::new(),
            filter_ext,
            running,
        }
    }

    fn fetch_links(&self, url: &str) -> Vec<String> {
        let response = match self.client.get(url).send() {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error fetching {}: {}", url, e);
                return Vec::new();
            }
        };

        if !response.status().is_success() {
            eprintln!("Error fetching {}: HTTP {}", url, response.status());
            return Vec::new();
        }

        let body = match response.text() {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Error reading response from {}: {}", url, e);
                return Vec::new();
            }
        };

        let base_url = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return Vec::new(),
        };

        let document = Html::parse_document(&body);
        let selector = Selector::parse("a").unwrap();

        let mut links = Vec::new();
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(full_url) = base_url.join(href) {
                    let full_url_str = full_url.to_string();
                    if full_url_str.starts_with("http://") || full_url_str.starts_with("https://") {
                        links.push(full_url_str);
                    }
                }
            }
        }

        links
    }

    fn is_valid_link(&self, link: &str) -> bool {
        match &self.filter_ext {
            Some(ext) => link.to_lowercase().ends_with(ext),
            None => true,
        }
    }

    fn is_directory_link(&self, link: &str) -> bool {
        if let Some(last_segment) = link.split('/').last() {
            link.contains('/') && !last_segment.contains('.')
        } else {
            false
        }
    }

    fn scrape_links(&mut self, url: &str, depth: u32) {
        if depth == 0 || self.visited_urls.contains(url) || !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.visited_urls.insert(url.to_string());

        let links = self.fetch_links(url);
        for link in links {
            if !self.running.load(Ordering::SeqCst) {
                return;
            }

            if self.is_valid_link(&link) || self.is_directory_link(&link) {
                if self.is_valid_link(&link) {
                    println!("{}", link);
                }
                self.scrape_links(&link, depth - 1);
            }
        }
    }

    fn run(&mut self, url: &str, max_depth: u32) {
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };

        let max_depth = if max_depth > 10 {
            println!("Depth too large. Limiting to 10 for performance reasons.");
            10
        } else {
            max_depth
        };

        if let Some(ref ext) = self.filter_ext {
            println!("Filter set to: {}", ext);
        }

        self.scrape_links(&url, max_depth);
    }
}

fn main() {
    let args = Args::parse();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("Quitting...");
        r.store(false, Ordering::SeqCst);
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut scraper = Scraper::new(args.filter, running);
    scraper.run(&args.url, args.max_depth);
}

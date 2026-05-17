# web-scraper

A command-line web scraper that recursively finds links on web pages.

## Build

```bash
cargo build --release
```

## Usage

```bash
web-scraper <url> [options]
```

### Options

| Flag | Description |
| ------ | ------------- |
| `-m, --max-depth <N>` | How deep to follow links (default: 1, max: 10) |
| `-f, --filter <ext>` | Only show links with this file extension |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

### Examples

```bash
# Scrape links from a page
web-scraper example.com

# Scrape 3 levels deep
web-scraper example.com -m 3

# Find all .pdf files
web-scraper example.com -m 5 -f pdf
```

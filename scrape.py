#!/usr/bin/env python

import signal
import sys
from bs4 import BeautifulSoup
import requests
import argparse
from urllib.parse import urljoin

class Scraper:
    def __init__(self):
        self.prog = 'scraper'
        self.version = '1.4.7'
        self.author = 'Al Biheiri (al@forgottheaddress.com)'
        self.http_timeout = 10
        self.visited_urls = set()
        self.filter_ext = None

    def fetch_links(self, url):
        """ Fetch links from a given URL and return full URLs. """
        try:
            response = requests.get(url, timeout=self.http_timeout)
            response.raise_for_status()
            soup = BeautifulSoup(response.content, features='lxml')

            links = []
            for link in soup.findAll('a'):
                href = link.get('href')
                if href:
                    full_url = urljoin(url, href)
                    if full_url.startswith(('http://', 'https://')):
                        links.append(full_url)

            return links
        except requests.RequestException as e:
            print(f"Error fetching {url}: {e}")
            return []

    def is_valid_link(self, link):
        """ Check if a link is valid based on the filter. """
        if self.filter_ext:
            return link.lower().endswith(self.filter_ext)
        return True

    def is_directory_link(self, link):
        """ Check if a link is likely a directory. """
        return '/' in link and '.' not in link.split('/')[-1]

    def scrape_links(self, url, depth):
        """ Recursively scrape links to a specified depth. """
        if depth <= 0 or url in self.visited_urls:
            return

        self.visited_urls.add(url)

        links = self.fetch_links(url)
        for link in links:
            if self.is_valid_link(link) or self.is_directory_link(link):
                if self.is_valid_link(link):
                    print(link)
                self.scrape_links(link, depth - 1)

    def run(self, url, max_depth):
        """ Run the scraper with the given arguments. """
        if not url.startswith(('http://', 'https://')):
            url = 'http://' + url

        if max_depth > 10:
            print("Depth too large. Limiting to 10 for performance reasons.")
            max_depth = 10

        if self.filter_ext:
            print(f"Filter set to: {self.filter_ext}")
        self.scrape_links(url, max_depth)

    def parse_args(self, args):
        """ Parse command line arguments. """
        parser = argparse.ArgumentParser(description="Web Scraper")
        parser.add_argument('url', help="URL to scrape")
        parser.add_argument('-m', '--max_depth', type=int, help="Maximum depth to scrape", default=1)
        parser.add_argument('-f', '--filter', type=str, help="Filter results by file extension, e.g., 'mkv' (without the dot)")
        parser.add_argument('--version', action='version', version=f'{self.prog} {self.version}')

        args = parser.parse_args(args)
        if args.filter:
            self.filter_ext = '.' + args.filter.lower()  # Ensure dot is included
        return args

def signal_handler(sig, frame):
    print("Quitting...")
    sys.exit(0)

if __name__ == "__main__":
    # Register the signal handler for Ctrl+C
    signal.signal(signal.SIGINT, signal_handler)

    scraper = Scraper()
    args = scraper.parse_args(sys.argv[1:])
    scraper.run(args.url, args.max_depth)

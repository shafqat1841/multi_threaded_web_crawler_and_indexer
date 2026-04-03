#The Mission
Build a system that simulates crawling a website. It has three distinct parts running at the same time:

The Producers (Crawlers): Multiple threads that "visit" a URL, wait (simulate network latency), and find "new" URLs.

The Coordinator (Shared State): A central record that keeps track of which URLs have already been visited so you don't crawl the same page twice.

The Consumer (Indexer): A single background thread that receives discovered data and "indexes" it (updates a global word count).

#The Requirements
Shared State: Use a HashSet protected by a Mutex to store visited URLs.

The "URL" Queue: Start with a Vec of 5 "Starting URLs" (just strings like "https://rust.org").

Crawler Threads: Spawn 4 threads. Each thread:

Pops a URL from the queue.

Checks the Mutex<HashSet> to see if it's been visited.

If not, adds it to the set and "crawls" it (sleep for 50ms).

"Discovers" 2 new fake URLs (e.g., "https://rust.org/page1") and adds them back to the queue.

Sends a Message { url: String, words: usize } through a channel.

Indexer Thread: A separate thread that listens to the Receiver. It maintains a total word count of everything crawled.

Graceful Exit: The program should stop after 50 total URLs have been visited.
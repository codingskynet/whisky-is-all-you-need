# whiskybase-crawler

This project is crawling whiskybase site(obeying its [robots.txt](https://en.wikipedia.org/wiki/Robots.txt)) and processing data.

Goal
- `[ ] Crawl all available whiskybase site data and save on persistence memory
  - [x] Crawl data on sitemap
  - [ ] Make server handling data with RDBMS and update on everyday
- [ ] Process each whisky's data
  - [ ] Merge similar or same product's score
  - [ ] Collect each user's score and erase bias

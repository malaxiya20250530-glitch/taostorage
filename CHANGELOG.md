# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-12

### Added
- SQLite storage engine via Node.js built-in `node:sqlite` (zero deps)
- Tag system with dedicated `tags` table and indexed queries
- Full-text fuzzy search across key, value, and tag fields
- Tag cloud with ASCII bar chart visualization
- Tag AND/OR combination queries
- Sorting support (`--sort`, `--order`)
- Pagination support (`--limit`)
- Transactional import with auto-rollback on failure
- Cascade delete for tags when items are removed
- Comprehensive statistics with top keys and top tags

### Changed
- Storage backend from JSON files to SQLite (WAL mode)
- Backup format includes tags, stats, and metadata
- CLI output now shows tags in all views

### Removed
- JSON file dependencies (`db.json`, `index.json` → single `tao.db`)

## [0.1.0] - 2026-06-12

### Added
- JSON file-based storage engine
- Basic CRUD: put, get, list, delete
- Key-based indexing with `keyIndex`
- Tag indexing with `tagIndex`
- Fuzzy search (in-memory substring match)
- Export / Import backup as JSON
- Initial CLI with 11 commands

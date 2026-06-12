# Contributing / 贡献指南

## 🌐 Language / 语言

- Issues and PRs can be written in **Chinese** or **English**
- Code comments should be in English
- README updates should be **bilingual** (Chinese + English)

## 🚀 Development Setup

```bash
git clone https://github.com/malaxiya20250530-glitch/taostorage.git
cd taostorage
# No npm install needed — zero dependencies!
node bin/tao.js init
```

## 📝 Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add tag cloud visualization
fix: correct cascade delete for tags
docs: bilingual README update
refactor: extract DB connection to db.js
test: add integration tests for put/get
chore: update .gitignore
```

## 🧪 Testing

```bash
# Manual E2E test
node bin/tao.js init
node bin/tao.js put test "hello" --tag demo
node bin/tao.js get test
node bin/tao.js list
node bin/tao.js search hello
node bin/tao.js tag demo
node bin/tao.js tags
node bin/tao.js export backup.json
node bin/tao.js import backup.json
node bin/tao.js stats
node bin/tao.js delete test
```

## 📦 Release Process

1. Update version in `package.json` and `config.json`
2. Update `CHANGELOG.md`
3. Create a tag: `git tag v0.x.x`
4. Push: `git push origin main --tags`
5. Create a GitHub Release with release notes

## 📁 Project Structure

```
taostorage/
├── bin/
│   └── tao.js          # CLI entry point
├── core/
│   ├── db.js           # SQLite connection
│   ├── store.js        # CRUD operations
│   ├── indexer.js      # Search & tag queries
│   ├── export.js       # Backup export
│   └── import.js       # Backup import
├── data/               # User data (gitignored)
├── .gitignore
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE
├── README.md
├── config.json
└── package.json
```

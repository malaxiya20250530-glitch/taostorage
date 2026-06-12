/**
 * db.js — TaoStorage v0.2 数据库连接
 *
 * 基于 Node.js 内置 node:sqlite，零依赖。
 */

import { DatabaseSync } from 'node:sqlite';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CONFIG = JSON.parse(fs.readFileSync(path.join(ROOT, 'config.json'), 'utf-8'));
const DB_PATH = path.resolve(ROOT, CONFIG.dbFile);

// 确保 data 目录存在
const dataDir = path.dirname(DB_PATH);
if (!fs.existsSync(dataDir)) {
  fs.mkdirSync(dataDir, { recursive: true });
}

let _db = null;

/**
 * 获取数据库连接（单例）
 */
export function getDB() {
  if (_db) return _db;

  _db = new DatabaseSync(DB_PATH);

  // 开启 WAL 模式（读写并发友好）
  _db.exec('PRAGMA journal_mode=WAL');
  _db.exec('PRAGMA foreign_keys=ON');

  return _db;
}

/**
 * 关闭数据库连接
 */
export function closeDB() {
  if (_db) {
    _db.close();
    _db = null;
  }
}

/**
 * 初始化数据库表结构
 */
export function initSchema() {
  const db = getDB();

  db.exec(`
    CREATE TABLE IF NOT EXISTS items (
      id         TEXT PRIMARY KEY,
      key        TEXT NOT NULL,
      value      TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS tags (
      item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
      tag     TEXT NOT NULL,
      PRIMARY KEY (item_id, tag)
    );

    CREATE INDEX IF NOT EXISTS idx_items_key ON items(key);
    CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
    CREATE INDEX IF NOT EXISTS idx_items_created ON items(created_at);
  `);
}

/**
 * 获取数据库路径
 */
export function getDBPath() {
  return DB_PATH;
}

/**
 * 重新打开数据库（导入时用）
 */
export function reopen() {
  closeDB();
  return getDB();
}

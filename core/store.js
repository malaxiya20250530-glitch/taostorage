/**
 * store.js — TaoStorage v0.2 存储引擎（SQLite）
 *
 * CRUD 操作：put / get / list / delete / stats
 */

import { randomUUID } from 'crypto';
import { getDB, initSchema } from './db.js';

/**
 * 初始化存储
 */
export function init() {
  initSchema();
  console.log(`📦 TaoStorage v0.2 (SQLite) 已初始化`);
}

/**
 * 写入数据
 * @param {string} key
 * @param {string} value
 * @param {string[]} [tags]
 * @returns {object} 创建的 item
 */
export function put(key, value, tags = []) {
  const db = getDB();
  const id = randomUUID();
  const now = Date.now();

  const insertItem = db.prepare(
    'INSERT INTO items (id, key, value, created_at, updated_at) VALUES (?, ?, ?, ?, ?)'
  );
  const insertTag = db.prepare(
    'INSERT OR IGNORE INTO tags (item_id, tag) VALUES (?, ?)'
  );

  // 手动事务
  db.exec('BEGIN');
  try {
    insertItem.run(id, key, value, now, now);
    for (const tag of tags) {
      insertTag.run(id, tag);
    }
    db.exec('COMMIT');
  } catch (e) {
    db.exec('ROLLBACK');
    throw e;
  }

  console.log(`✅ 已存储 [${key}] → ${id.slice(0, 8)}...${tags.length > 0 ? ` [${tags.join(', ')}]` : ''}`);
  return { id, key, value, tags, createdAt: now, updatedAt: now };
}

/**
 * 按 key 查询
 * @param {string} key
 * @returns {object[]}
 */
export function get(key) {
  const db = getDB();
  const rows = db.prepare('SELECT * FROM items WHERE key = ? ORDER BY created_at DESC').all(key);
  return rows.map(r => rowToItem(r));
}

/**
 * 按 ID 查询
 * @param {string} id
 * @returns {object|null}
 */
export function getById(id) {
  const db = getDB();
  const row = db.prepare('SELECT * FROM items WHERE id = ?').get(id);
  return row ? rowToItem(row) : null;
}

/**
 * 按 tag 查询
 * @param {string} tag
 * @returns {object[]}
 */
export function getByTag(tag) {
  const db = getDB();
  const rows = db.prepare(`
    SELECT i.* FROM items i
    JOIN tags t ON t.item_id = i.id
    WHERE t.tag = ?
    ORDER BY i.created_at DESC
  `).all(tag);
  return rows.map(r => rowToItem(r));
}

/**
 * 列出所有数据
 * @param {object} [opts]
 * @returns {object[]}
 */
export function list(opts = {}) {
  const db = getDB();
  let sql = 'SELECT i.* FROM items i';
  const params = [];

  if (opts.tag) {
    sql += ' JOIN tags t ON t.item_id = i.id WHERE t.tag = ?';
    params.push(opts.tag);
  } else if (opts.key) {
    sql += ' WHERE i.key = ?';
    params.push(opts.key);
  }

  const orderBy = opts.orderBy || 'created_at';
  const order = (opts.order || 'DESC').toUpperCase() === 'ASC' ? 'ASC' : 'DESC';
  sql += ` ORDER BY i.${orderBy} ${order}`;

  if (opts.limit) {
    sql += ' LIMIT ?';
    params.push(opts.limit);
  }
  if (opts.offset) {
    sql += ' OFFSET ?';
    params.push(opts.offset);
  }

  const rows = db.prepare(sql).all(...params);
  return rows.map(r => rowToItem(r));
}

/**
 * 删除指定 key 的数据
 * @param {string} key
 * @returns {number}
 */
export function remove(key) {
  const db = getDB();
  const result = db.prepare('DELETE FROM items WHERE key = ?').run(key);
  const count = result.changes;
  console.log(`🗑️  已删除 ${count} 条 [${key}]`);
  return count;
}

/**
 * 删除指定 ID 的数据
 * @param {string} id
 * @returns {boolean}
 */
export function removeById(id) {
  const db = getDB();
  const result = db.prepare('DELETE FROM items WHERE id = ?').run(id);
  return result.changes > 0;
}

/**
 * 统计信息
 * @returns {object}
 */
export function stats() {
  const db = getDB();
  const totalItems = db.prepare('SELECT COUNT(*) as c FROM items').get().c;
  const uniqueKeys = db.prepare('SELECT COUNT(DISTINCT key) as c FROM items').get().c;
  const totalTags = db.prepare('SELECT COUNT(DISTINCT tag) as c FROM tags').get().c;
  const topKeys = db.prepare(
    'SELECT key, COUNT(*) as count FROM items GROUP BY key ORDER BY count DESC LIMIT 5'
  ).all();
  const topTags = db.prepare(
    'SELECT tag, COUNT(*) as count FROM tags GROUP BY tag ORDER BY count DESC LIMIT 5'
  ).all();

  return { totalItems, uniqueKeys, totalTags, topKeys, topTags };
}

/**
 * 获取所有数据（用于导出）
 */
export function getAll() {
  const db = getDB();
  const items = db.prepare('SELECT * FROM items ORDER BY created_at ASC').all();
  const tags = db.prepare('SELECT * FROM tags ORDER BY item_id, tag').all();
  return { items: items.map(r => rowToItem(r)), tags };
}

/**
 * 批量导入（事务）
 * @param {{ items: object[], tags: {item_id:string, tag:string}[] }} data
 */
export function bulkImport(data) {
  const db = getDB();

  const insertItem = db.prepare(
    'INSERT OR REPLACE INTO items (id, key, value, created_at, updated_at) VALUES (?, ?, ?, ?, ?)'
  );
  const insertTag = db.prepare(
    'INSERT OR IGNORE INTO tags (item_id, tag) VALUES (?, ?)'
  );

  db.exec('BEGIN');
  try {
    db.exec('DELETE FROM tags');
    db.exec('DELETE FROM items');

    for (const item of data.items) {
      insertItem.run(
        item.id, item.key, item.value,
        item.createdAt || item.created_at,
        item.updatedAt || item.updated_at
      );
    }
    for (const t of data.tags) {
      insertTag.run(t.item_id, t.tag);
    }
    db.exec('COMMIT');
  } catch (e) {
    db.exec('ROLLBACK');
    throw e;
  }
}

// ─── 内部帮助 ────────────────────────────────────────────

function rowToItem(row) {
  const db = getDB();
  const tagRows = db.prepare('SELECT tag FROM tags WHERE item_id = ? ORDER BY tag').all(row.id);
  return {
    id: row.id,
    key: row.key,
    value: row.value,
    tags: tagRows.map(t => t.tag),
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

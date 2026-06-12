/**
 * indexer.js — TaoStorage v0.2 高级查询引擎
 *
 * 基于 SQLite 的全文搜索、tag 组合筛选、时间范围查询。
 */

import { getDB } from './db.js';

/**
 * 模糊搜索（key / value / tag 全字段匹配）
 * @param {string} query
 * @returns {object[]}
 */
export function search(query) {
  const db = getDB();
  const like = `%${query}%`;

  const rows = db.prepare(`
    SELECT DISTINCT i.* FROM items i
    LEFT JOIN tags t ON t.item_id = i.id
    WHERE i.key LIKE ? OR i.value LIKE ? OR t.tag LIKE ?
    ORDER BY i.created_at DESC
  `).all(like, like, like);

  return rows.map(rowToItem);
}

/**
 * 按时间范围查询
 * @param {number} startMs - 起始时间戳（毫秒）
 * @param {number} endMs - 结束时间戳（毫秒）
 * @returns {object[]}
 */
export function byTimeRange(startMs, endMs) {
  const db = getDB();
  const rows = db.prepare(
    'SELECT * FROM items WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC'
  ).all(startMs, endMs);
  return rows.map(rowToItem);
}

/**
 * 按多个 tag 查询（AND 逻辑）
 * @param {string[]} tags - tag 列表
 * @returns {object[]}
 */
export function byTagsAnd(tags) {
  if (tags.length === 0) return [];
  const db = getDB();

  const placeholders = tags.map(() => '?').join(',');
  const rows = db.prepare(`
    SELECT i.* FROM items i
    JOIN tags t ON t.item_id = i.id
    WHERE t.tag IN (${placeholders})
    GROUP BY i.id
    HAVING COUNT(DISTINCT t.tag) = ?
    ORDER BY i.created_at DESC
  `).all(...tags, tags.length);

  return rows.map(rowToItem);
}

/**
 * 按多个 tag 查询（OR 逻辑）
 * @param {string[]} tags
 * @returns {object[]}
 */
export function byTagsOr(tags) {
  if (tags.length === 0) return [];
  const db = getDB();

  const placeholders = tags.map(() => '?').join(',');
  const rows = db.prepare(`
    SELECT DISTINCT i.* FROM items i
    JOIN tags t ON t.item_id = i.id
    WHERE t.tag IN (${placeholders})
    ORDER BY i.created_at DESC
  `).all(...tags);

  return rows.map(rowToItem);
}

/**
 * 获取所有 tag 及其计数
 * @returns {{ tag: string, count: number }[]}
 */
export function tagCloud() {
  const db = getDB();
  return db.prepare(
    'SELECT tag, COUNT(*) as count FROM tags GROUP BY tag ORDER BY count DESC'
  ).all();
}

/**
 * 按 key 分组统计
 * @returns {{ key: string, count: number }[]}
 */
export function keyGroups() {
  const db = getDB();
  return db.prepare(
    'SELECT key, COUNT(*) as count FROM items GROUP BY key ORDER BY count DESC'
  ).all();
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

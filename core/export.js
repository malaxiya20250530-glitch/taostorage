/**
 * export.js — TaoStorage v0.2 导出引擎
 *
 * 将 SQLite 数据导出为可移植的 JSON 备份文件。
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { getAll, stats } from './store.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CONFIG = JSON.parse(fs.readFileSync(path.join(ROOT, 'config.json'), 'utf-8'));

/**
 * 导出完整备份
 * @param {string} outputPath
 */
export function exportBackup(outputPath) {
  const data = getAll();
  const s = stats();

  const backup = {
    version: CONFIG.version,
    exportedAt: Date.now(),
    stats: s,
    data,
  };

  const resolvedPath = path.resolve(outputPath);
  fs.writeFileSync(resolvedPath, JSON.stringify(backup, null, 2));

  console.log(`📤 已导出到 ${resolvedPath}`);
  console.log(`   共 ${data.items.length} 条数据, ${data.tags.length} 个标签`);
}

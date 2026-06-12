/**
 * import.js — TaoStorage v0.2 导入引擎
 *
 * 从备份 JSON 恢复数据到 SQLite。
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { bulkImport } from './store.js';
import { reopen } from './db.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

/**
 * 导入备份文件（覆盖现有数据）
 * @param {string} inputPath
 */
export function importBackup(inputPath) {
  const resolvedPath = path.resolve(inputPath);

  if (!fs.existsSync(resolvedPath)) {
    console.error(`❌ 备份文件不存在: ${resolvedPath}`);
    process.exit(1);
  }

  const raw = fs.readFileSync(resolvedPath, 'utf-8');
  let backup;

  try { backup = JSON.parse(raw); }
  catch (e) {
    console.error('❌ 备份文件格式错误（不是有效的 JSON）');
    process.exit(1);
  }

  if (!backup.data || !backup.data.items) {
    console.error('❌ 备份文件格式错误（缺少 data.items）');
    process.exit(1);
  }

  // 导入前重新打开数据库，确保连接正常
  reopen();

  bulkImport(backup.data);

  console.log(`📥 已从 ${resolvedPath} 导入`);
  console.log(`   共 ${backup.data.items.length} 条数据`);
  if (backup.exportedAt) {
    console.log(`   导出时间: ${new Date(backup.exportedAt).toLocaleString()}`);
  }
}

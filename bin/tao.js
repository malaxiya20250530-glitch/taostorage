#!/usr/bin/env node

/**
 * tao.js — TaoStorage v0.2 CLI 入口（SQLite 版）
 *
 * 用法:
 *   tao init               初始化
 *   tao put <key> <val>    写入（可用 --tag a,b,c）
 *   tao get <key>          读取
 *   tao list               浏览
 *   tao delete <key>       删除
 *   tao search <query>     模糊搜索
 *   tao tag <tagname>      按标签查询
 *   tao tags               标签云
 *   tao export [path]      导出
 *   tao import <path>      导入
 *   tao stats              统计
 *   tao help               帮助
 */

// ─── 解析命令行参数 ───────────────────────────────────────

function parseArgs(argv) {
  const args = [];
  const opts = {};

  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith('--')) {
      const eqIdx = a.indexOf('=');
      if (eqIdx !== -1) {
        opts[a.slice(2, eqIdx)] = a.slice(eqIdx + 1);
      } else {
        const val = argv[i + 1];
        if (val !== undefined && !val.startsWith('--')) {
          opts[a.slice(2)] = val;
          i++;
        } else {
          opts[a.slice(2)] = true;
        }
      }
    } else {
      args.push(a);
    }
  }

  return { args, opts };
}

const { args, opts } = parseArgs(process.argv.slice(2));
const cmd = args[0];

// ─── 导入模块 ─────────────────────────────────────────────

const store = await import('../core/store.js');
const indexer = await import('../core/indexer.js');
const expmod = await import('../core/export.js');
const impmod = await import('../core/import.js');

// ─── 格式化输出 ───────────────────────────────────────────

function formatItem(item, verbose = false) {
  const date = new Date(item.updatedAt).toLocaleString();
  const tagStr = item.tags && item.tags.length > 0
    ? ` [${item.tags.join(', ')}]`
    : '';
  const idStr = verbose ? ` (${item.id.slice(0, 8)}...)` : '';
  return `  ${item.key}${idStr}${tagStr}\n    ${item.value}\n    📅 ${date}`;
}

function printTable(items) {
  if (items.length === 0) {
    console.log('  (空)');
    return;
  }

  const maxKeyLen = Math.max(...items.map(i => i.key.length), 4);
  const maxTagLen = Math.max(...items.map(i => (i.tags || []).join(',').length), 4);

  console.log(`  ${'KEY'.padEnd(maxKeyLen)}  ${'TAGS'.padEnd(maxTagLen)}  VALUE`);
  console.log(`  ${'─'.repeat(maxKeyLen)}  ${'─'.repeat(maxTagLen)}  ${'─'.repeat(50)}`);

  for (const item of items) {
    const val = item.value.length > 47
      ? item.value.slice(0, 44) + '...'
      : item.value;
    const tagStr = (item.tags || []).join(',');
    console.log(`  ${item.key.padEnd(maxKeyLen)}  ${tagStr.padEnd(maxTagLen)}  ${val}`);
  }
}

// ─── 命令分发 ─────────────────────────────────────────────

switch (cmd) {

  case 'init': {
    store.init();
    break;
  }

  case 'put': {
    const key = args[1];
    const value = args.slice(2).join(' ');
    if (!key || !value) {
      console.error('❌ 用法: tao put <key> <value> [--tag tag1,tag2]');
      process.exit(1);
    }
    const tags = opts.tag ? opts.tag.split(',').map(t => t.trim()).filter(Boolean) : [];
    store.put(key, value, tags);
    break;
  }

  case 'get': {
    const key = args[1];
    if (!key) {
      console.error('❌ 用法: tao get <key>');
      process.exit(1);
    }
    const items = store.get(key);
    if (items.length === 0) {
      console.log(`🔍 未找到 [${key}]`);
    } else {
      console.log(`🔍 找到 ${items.length} 条 [${key}]:\n`);
      for (const item of items) {
        console.log(formatItem(item, true));
        console.log('');
      }
    }
    break;
  }

  case 'list': {
    const key = opts.key || null;
    const tag = opts.tag || null;
    const limit = opts.limit ? parseInt(opts.limit) : null;
    const orderBy = opts.sort || 'created_at';
    const order = opts.order || 'DESC';

    const items = store.list({ key, tag, limit, orderBy, order });
    if (items.length === 0) {
      console.log('📭 暂无数据');
    } else {
      console.log(`📋 共 ${items.length} 条数据:\n`);
      printTable(items);
    }
    break;
  }

  case 'delete': {
    const key = args[1];
    if (!key) {
      console.error('❌ 用法: tao delete <key>');
      process.exit(1);
    }
    store.remove(key);
    break;
  }

  case 'search': {
    const query = args[1];
    if (!query) {
      console.error('❌ 用法: tao search <query>');
      process.exit(1);
    }
    const results = indexer.search(query);
    if (results.length === 0) {
      console.log(`🔍 未找到包含 "${query}" 的数据`);
    } else {
      console.log(`🔍 找到 ${results.length} 条:\n`);
      for (const item of results) {
        console.log(formatItem(item));
        console.log('');
      }
    }
    break;
  }

  case 'tag': {
    const tag = args[1];
    if (!tag) {
      console.error('❌ 用法: tao tag <tagname>');
      process.exit(1);
    }
    const items = store.getByTag(tag);
    if (items.length === 0) {
      console.log(`🔖 未找到 tag [${tag}]`);
    } else {
      console.log(`🔖 tag [${tag}] → ${items.length} 条:\n`);
      for (const item of items) {
        console.log(formatItem(item));
        console.log('');
      }
    }
    break;
  }

  case 'tags': {
    const cloud = indexer.tagCloud();
    if (cloud.length === 0) {
      console.log('🔖 暂无标签');
    } else {
      console.log('☁️  标签云:\n');
      const maxCount = Math.max(...cloud.map(t => t.count));
      for (const t of cloud) {
        const bar = '█'.repeat(Math.round((t.count / maxCount) * 20));
        console.log(`  ${t.tag.padEnd(16)} ${String(t.count).padStart(3)}  ${bar}`);
      }
    }
    break;
  }

  case 'export': {
    const outputPath = args[1] || './tao-backup.json';
    expmod.exportBackup(outputPath);
    break;
  }

  case 'import': {
    const inputPath = args[1];
    if (!inputPath) {
      console.error('❌ 用法: tao import <backup.json>');
      process.exit(1);
    }
    impmod.importBackup(inputPath);
    break;
  }

  case 'stats': {
    const s = store.stats();
    console.log(`📊 TaoStorage v0.2 统计\n`);
    console.log(`  总数据条数:   ${s.totalItems}`);
    console.log(`  唯一 Key 数:  ${s.uniqueKeys}`);
    console.log(`  总标签数:     ${s.totalTags}`);
    if (s.topKeys.length > 0) {
      console.log(`\n  🔑 热门 Key:`);
      for (const k of s.topKeys) {
        console.log(`    ${k.key.padEnd(16)} ${k.count}`);
      }
    }
    if (s.topTags.length > 0) {
      console.log(`\n  🏷️  热门标签:`);
      for (const t of s.topTags) {
        console.log(`    ${t.tag.padEnd(16)} ${t.count}`);
      }
    }
    break;
  }

  case 'help':
  case '--help':
  case '-h':
  case undefined: {
    const usage = `
  🦀 TaoStorage v0.2 — 个人数据仓库 CLI（SQLite）

  用法:
    tao init                        初始化 SQLite 存储
    tao put <key> <val> [--tag a,b] 写入数据（可加标签）
    tao get <key>                   按 key 读取
    tao list [--key x] [--tag t]    浏览数据
             [--limit n] [--sort field] [--order ASC|DESC]
    tao delete <key>                删除 key
    tao search <query>              模糊搜索（key/value/tag）
    tao tag <tagname>               按标签查询
    tao tags                        标签云
    tao export [path]               导出备份 JSON
    tao import <path>               导入恢复
    tao stats                       统计信息
    tao help                        显示帮助

  示例:
    tao init
    tao put note "道可道" --tag philosophy,chinese
    tao put todo "买牛奶" --tag shopping
    tao put code "hello world" --tag javascript,example
    tao get note
    tao list
    tao search 道
    tao tag shopping
    tao tags
    tao stats
    tao export backup.json
`;
    console.log(usage);
    break;
  }

  default: {
    console.error(`❌ 未知命令: ${cmd}`);
    console.error('   可用: init, put, get, list, delete, search, tag, tags, export, import, stats, help');
    process.exit(1);
  }
}

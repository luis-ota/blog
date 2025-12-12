#!/usr/bin/env bun
const fs = require("fs");
const path = require("path");

function humanizeSlug(slug) {
  return slug.replace(/[-_]+/g, " ").replace(/\b\w/g, (m) => m.toUpperCase());
}

function safeReadFile(filePath) {
  try {
    return fs.readFileSync(filePath, "utf8");
  } catch (e) {
    return null;
  }
}

function extractTitleFromMarkdown(content) {
  if (!content) return null;

  const fmMatch = content.match(/^---\s*[\s\S]*?^title:\s*(.+)\s*$/m);
  if (fmMatch && fmMatch[1]) {
    return fmMatch[1].trim().replace(/^['"]|['"]$/g, "");
  }

  const h1 = content.match(/^\s*#\s+(.+)\s*$/m);
  if (h1 && h1[1]) return h1[1].trim();

  const h2 = content.match(/^\s*##\s+(.+)\s*$/m);
  if (h2 && h2[1]) return h2[1].trim();

  return null;
}

function findMarkdownFileInDir(dirPath, preferredName = null) {
  if (preferredName) {
    const preferred = path.join(dirPath, preferredName);
    if (fs.existsSync(preferred) && fs.statSync(preferred).isFile())
      return preferred;
  }

  if (!fs.existsSync(dirPath)) return null;
  const candidates = fs
    .readdirSync(dirPath)
    .filter((f) => f.toLowerCase().endsWith(".md"));
  if (candidates.length === 0) return null;

  const index = candidates.find((c) => c.toLowerCase() === "index.md");
  if (index) return path.join(dirPath, index);
  return path.join(dirPath, candidates[0]);
}

function gatherPosts(postsDir) {
  const results = [];
  if (!fs.existsSync(postsDir)) return results;

  const entries = fs.readdirSync(postsDir, { withFileTypes: true });

  for (const entry of entries) {
    if (entry.isFile() && entry.name.toLowerCase().endsWith(".md")) {
      const fileName = entry.name;
      const slug = path.basename(fileName, path.extname(fileName));
      const filePath = path.join(postsDir, fileName);
      const content = safeReadFile(filePath) || "";
      const title = extractTitleFromMarkdown(content) || humanizeSlug(slug);
      results.push({ slug, title });
    }
  }

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const slug = entry.name;
    const dirPath = path.join(postsDir, slug);

    // try preferred <slug>.md first
    const preferred = `${slug}.md`;
    const mdPath = findMarkdownFileInDir(dirPath, preferred);

    if (mdPath) {
      const content = safeReadFile(mdPath) || "";
      const title = extractTitleFromMarkdown(content) || humanizeSlug(slug);
      results.push({ slug, title });
    } else {
      try {
        const dirFiles = fs.readdirSync(dirPath);
        if (dirFiles.length > 0) {
          results.push({ slug, title: humanizeSlug(slug) });
        }
      } catch (e) {
        // ignore unreadable directories
      }
    }
  }

  const seen = new Set();
  const deduped = [];
  for (const p of results) {
    if (!seen.has(p.slug)) {
      seen.add(p.slug);
      deduped.push(p);
    }
  }

  deduped.sort((a, b) => {
    const t = a.title.localeCompare(b.title, undefined, {
      sensitivity: "base",
    });
    if (t !== 0) return t;
    return a.slug.localeCompare(b.slug, undefined, { sensitivity: "base" });
  });

  return deduped;
}

function writeIfChanged(outPath, dataObj) {
  const newJson = JSON.stringify(dataObj, null, 2) + "\n";
  let existing = null;
  try {
    existing = fs.readFileSync(outPath, "utf8");
  } catch (e) {
    existing = null;
  }

  if (existing === newJson) {
    console.log(`No changes: ${outPath} unchanged (${dataObj.length} entries)`);
    return { written: false };
  }

  try {
    fs.writeFileSync(outPath, newJson, "utf8");
    console.log(`Wrote ${outPath} (${dataObj.length} entries)`);
    return { written: true };
  } catch (err) {
    console.error(
      `Failed to write ${outPath}:`,
      err && err.message ? err.message : err,
    );
    throw err;
  }
}

function main() {
  try {
    const repoRoot = path.resolve(__dirname, ".."); // blog/
    const postsDir = path.join(repoRoot, "posts");
    const outPath = path.join(repoRoot, "posts.json");

    const posts = gatherPosts(postsDir);
    const result = writeIfChanged(outPath, posts);
    process.exitCode = 0;
    return result;
  } catch (err) {
    console.error(
      "Error generating posts.json:",
      err && err.message ? err.message : err,
    );
    process.exitCode = 2;
    return null;
  }
}

if (require.main === module) {
  main();
}

module.exports = { gatherPosts, writeIfChanged, main };

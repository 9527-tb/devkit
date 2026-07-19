/**
 * 日志行格式化：ANSI 颜色、链接识别 → 安全 HTML。
 * 依赖：无；被 LogsPanel / 工作台控制台使用。
 * 对应 DESIGN.md §12.2 shared/logFormat
 */

// TODO(fe-panel-logs): 随 LogsPanel 迁出后由面板独占消费 — DESIGN §8

const FG = {
  30: "#6b7280",
  31: "#f87171",
  32: "#4ade80",
  33: "#facc15",
  34: "#60a5fa",
  35: "#c084fc",
  36: "#22d3ee",
  37: "#e5e7eb",
  90: "#9ca3af",
  91: "#fca5a5",
  92: "#86efac",
  93: "#fde047",
  94: "#93c5fd",
  95: "#d8b4fe",
  96: "#67e8f9",
  97: "#ffffff",
};

const BG = {
  40: "#111827",
  41: "#7f1d1d",
  42: "#14532d",
  43: "#713f12",
  44: "#1e3a8a",
  45: "#581c87",
  46: "#164e63",
  47: "#374151",
  100: "#1f2937",
  101: "#991b1b",
  102: "#166534",
  103: "#a16207",
  104: "#1d4ed8",
  105: "#7e22ce",
  106: "#0e7490",
  107: "#4b5563",
};

const URL_RE = /https?:\/\/[^\s<>"'`\\]+/g;
const ANSI_RE = /\u001b\[([0-9;]*)m/g;
const OTHER_ESC_RE = /\u001b\][^\u0007\u001b]*(?:\u0007|\u001b\\)|\u001b\[[0-9;?]*[ -/]*[@-ln-zA-Z]|\u001b[@-Z\\-_]|\r/g;

function escapeHtml(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function styleFromCodes(codes, state) {
  let i = 0;
  while (i < codes.length) {
    const code = codes[i];
    if (code === 0 || Number.isNaN(code)) {
      state.bold = false;
      state.dim = false;
      state.italic = false;
      state.underline = false;
      state.fg = "";
      state.bg = "";
      i += 1;
      continue;
    }
    if (code === 1) state.bold = true;
    else if (code === 2) state.dim = true;
    else if (code === 3) state.italic = true;
    else if (code === 4) state.underline = true;
    else if (code === 22) {
      state.bold = false;
      state.dim = false;
    } else if (code === 23) state.italic = false;
    else if (code === 24) state.underline = false;
    else if (code === 39) state.fg = "";
    else if (code === 49) state.bg = "";
    else if (code === 38 && codes[i + 1] === 5 && codes[i + 2] != null) {
      state.fg = `color:#${ansi256ToHex(codes[i + 2])}`;
      i += 3;
      continue;
    } else if (code === 38 && codes[i + 1] === 2 && codes[i + 4] != null) {
      state.fg = `color:rgb(${codes[i + 2]},${codes[i + 3]},${codes[i + 4]})`;
      i += 5;
      continue;
    } else if (code === 48 && codes[i + 1] === 5 && codes[i + 2] != null) {
      state.bg = `background:#${ansi256ToHex(codes[i + 2])}`;
      i += 3;
      continue;
    } else if (code === 48 && codes[i + 1] === 2 && codes[i + 4] != null) {
      state.bg = `background:rgb(${codes[i + 2]},${codes[i + 3]},${codes[i + 4]})`;
      i += 5;
      continue;
    } else if (FG[code]) state.fg = `color:${FG[code]}`;
    else if (BG[code]) state.bg = `background:${BG[code]}`;
    i += 1;
  }
}

function ansi256ToHex(n) {
  const c = Number(n) || 0;
  if (c < 16) {
    const basic = [
      "000000", "800000", "008000", "808000", "000080", "800080", "008080", "c0c0c0",
      "808080", "ff0000", "00ff00", "ffff00", "0000ff", "ff00ff", "00ffff", "ffffff",
    ];
    return basic[c] || "e5e7eb";
  }
  if (c >= 232) {
    const v = Math.max(0, Math.min(255, 8 + (c - 232) * 10)).toString(16).padStart(2, "0");
    return `${v}${v}${v}`;
  }
  const idx = c - 16;
  const r = Math.floor(idx / 36);
  const g = Math.floor((idx % 36) / 6);
  const b = idx % 6;
  const to = (x) => (x === 0 ? 0 : 55 + x * 40).toString(16).padStart(2, "0");
  return `${to(r)}${to(g)}${to(b)}`;
}

function openSpan(state) {
  const styles = [];
  if (state.fg) styles.push(state.fg);
  if (state.bg) styles.push(state.bg);
  if (state.bold) styles.push("font-weight:700");
  if (state.dim) styles.push("opacity:0.75");
  if (state.italic) styles.push("font-style:italic");
  if (state.underline) styles.push("text-decoration:underline");
  if (!styles.length) return "";
  return `<span style="${styles.join(";")}">`;
}

function linkifyEscaped(text) {
  URL_RE.lastIndex = 0;
  let out = "";
  let last = 0;
  let match;
  while ((match = URL_RE.exec(text))) {
    out += text.slice(last, match.index);
    let url = match[0];
    // Trim common trailing punctuation from log lines.
    url = url.replace(/[),.\];:'"]+$/g, "");
    const trailing = match[0].slice(url.length);
    out += `<a href="${url}" class="log-link" target="_self" rel="noopener" title="Ctrl/⌘ + 点击打开">${url}</a>${trailing}`;
    last = match.index + match[0].length;
  }
  out += text.slice(last);
  return out;
}

function wrapStyled(text, state) {
  const body = linkifyEscaped(escapeHtml(text));
  const open = openSpan(state);
  if (!open) return body;
  return `${open}${body}</span>`;
}

/**
 * 日志级别粗分类（供过滤 / 摘要）。
 * @returns {"info"|"warn"|"error"}
 */
export function classifyLine(line) {
  const s = String(line || "")
    .replace(/\u001b\[[0-9;?]*[ -/]*[@-~]/g, "")
    .replace(/\r/g, "");
  const lower = s.toLowerCase();
  if (
    lower.includes("eaddrinuse") ||
    lower.includes("address already in use") ||
    lower.includes("exception") ||
    lower.includes("fatal") ||
    lower.includes("npm err!") ||
    lower.includes("compilation failure") ||
    lower.includes("build failed") ||
    /(^|[^a-z])error([^a-z]|$)/i.test(s) ||
    lower.includes("failed") ||
    lower.includes("failure")
  ) {
    // 成功结束不算错误
    if (lower.includes("[devkit]") && lower.includes("exit 0")) return "info";
    return "error";
  }
  if (
    lower.includes("warn") ||
    lower.includes("warning") ||
    lower.includes("deprecated")
  ) {
    return "warn";
  }
  return "info";
}

/** Convert a log line with ANSI colors into safe HTML (with http links). */
export function formatLogLine(line) {
  const cleaned = String(line || "").replace(OTHER_ESC_RE, "");
  const state = {
    bold: false,
    dim: false,
    italic: false,
    underline: false,
    fg: "",
    bg: "",
  };
  let html = "";
  let last = 0;
  ANSI_RE.lastIndex = 0;
  let match;
  while ((match = ANSI_RE.exec(cleaned))) {
    if (match.index > last) {
      html += wrapStyled(cleaned.slice(last, match.index), state);
    }
    const codes = (match[1] || "0")
      .split(";")
      .filter((x) => x !== "")
      .map((x) => Number(x));
    if (!codes.length) codes.push(0);
    styleFromCodes(codes, state);
    last = match.index + match[0].length;
  }
  if (last < cleaned.length) {
    html += wrapStyled(cleaned.slice(last), state);
  }
  return html || "\u00a0";
}

export function formatLogHtml(lines) {
  return (lines || []).map(formatLogLine).join("\n");
}

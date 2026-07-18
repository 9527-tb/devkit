/**
 * 界面国际化：语言选项、文案表与翻译函数。
 * 依赖：locales/*；被 App / 各 feature 使用。
 * 对应 DESIGN.md §12.2 i18n/
 *
 * DONE(fe-i18n-complete): locales 拆分；界面文案走 t() — DESIGN §17 R7
 */

import zhCN from "./locales/zh-CN.js";
import zhTW from "./locales/zh-TW.js";
import en from "./locales/en.js";
import ja from "./locales/ja.js";

export const LOCALE_OPTIONS = [
  { id: "zh-CN", label: "简体中文" },
  { id: "zh-TW", label: "繁體中文" },
  { id: "en", label: "English" },
  { id: "ja", label: "日本語" },
];

const messages = {
  "zh-CN": zhCN,
  "zh-TW": zhTW,
  en,
  ja,
};

export function createTranslator(localeRef) {
  return (key, vars = {}) => {
    const locale =
      localeRef && typeof localeRef === "object" && "value" in localeRef
        ? localeRef.value
        : localeRef || "zh-CN";
    const table = messages[locale] || messages["zh-CN"];
    let text = table[key] ?? messages["zh-CN"][key] ?? key;
    for (const [k, v] of Object.entries(vars)) {
      text = text.replaceAll(`{${k}}`, String(v));
    }
    return text;
  };
}

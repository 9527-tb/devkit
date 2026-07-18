/**
 * 全局项目过滤：按 Kind 下列出的命令 checkbox 过滤侧栏。
 * 勾选 = 过滤条件（过滤掉不具备该命令的项目）；全不选 = 该类型不过滤。
 * 多选：项目具备其中任一勾选命令即保留（OR）。默认全部勾选。
 * Maven 另支持 onlyJar：仅保留 packaging=jar（未声明 packaging 视为 jar）。
 */

/**
 * @typedef {object} KindFilterRule
 * @property {string[]} actions 已勾选的命令 id
 * @property {boolean} [onlyJar] Maven：仅保留 packaging=jar
 */

/** @typedef {Record<string, KindFilterRule>} ProjectFilter */

export const FILTER_KINDS = ["maven", "node", "cargo", "gradle"];

/** 各 Kind 可勾选的命令（设置页直接列出） */
export const FILTER_ACTION_OPTIONS = {
  maven: [
    { value: "spring-boot:run", label: "spring-boot:run" },
    { value: "package", label: "package" },
    { value: "install", label: "install" },
    { value: "clean install", label: "clean install" },
    { value: "clean", label: "clean" },
  ],
  node: [
    { value: "script:start", label: "start" },
    { value: "script:dev", label: "dev" },
    { value: "script:build", label: "build" },
    { value: "script:lint", label: "lint" },
    { value: "install", label: "install" },
  ],
  cargo: [
    { value: "run", label: "run" },
    { value: "build", label: "build" },
    { value: "test", label: "test" },
    { value: "check", label: "check" },
    { value: "clean", label: "clean" },
  ],
  gradle: [
    { value: "bootRun", label: "bootRun" },
    { value: "build", label: "build" },
    { value: "test", label: "test" },
    { value: "clean", label: "clean" },
  ],
};

export function allActionValues(kind) {
  return (FILTER_ACTION_OPTIONS[kind] || []).map((x) => x.value);
}

/** 默认：该类型全部命令勾选；Maven 默认仅保留 jar */
export function defaultKindRule(kind) {
  if (kind === "maven") {
    return { actions: allActionValues(kind), onlyJar: true };
  }
  return { actions: allActionValues(kind), onlyJar: false };
}

export function emptyProjectFilter() {
  return {
    maven: defaultKindRule("maven"),
    node: defaultKindRule("node"),
    cargo: defaultKindRule("cargo"),
    gradle: defaultKindRule("gradle"),
  };
}

/**
 * @param {string} kind
 * @param {unknown} raw
 * @returns {KindFilterRule}
 */
function normalizeKindRule(kind, raw) {
  const allowed = new Set(allActionValues(kind));
  const fallback = defaultKindRule(kind);

  let actions = [...fallback.actions];
  let onlyJar = fallback.onlyJar;

  if (raw && typeof raw === "object" && Array.isArray(raw.actions)) {
    actions = raw.actions.filter((a) => typeof a === "string" && allowed.has(a));
    if (kind === "maven") {
      // 缺省字段：新装/旧配置无 onlyJar 时默认开启 jar 过滤
      onlyJar = typeof raw.onlyJar === "boolean" ? raw.onlyJar : true;
    } else {
      onlyJar = false;
    }
    return { actions, onlyJar };
  }
  if (raw && typeof raw === "object" && typeof raw.action === "string") {
    const mode = raw.mode;
    if (mode === "all" || !raw.action) {
      actions = [];
    } else if (allowed.has(raw.action)) {
      if (mode === "lacks") actions = [];
      else actions = [raw.action];
    }
    if (kind === "maven") {
      onlyJar = typeof raw.onlyJar === "boolean" ? raw.onlyJar : true;
    }
    return { actions, onlyJar };
  }
  return { ...fallback, actions: [...fallback.actions] };
}

/**
 * @param {unknown} raw
 * @returns {ProjectFilter}
 */
export function normalizeProjectFilter(raw) {
  const out = emptyProjectFilter();
  if (!raw || typeof raw !== "object") return out;
  for (const kind of FILTER_KINDS) {
    out[kind] = normalizeKindRule(kind, raw[kind]);
  }
  return out;
}

/** 与工作台 ActionBar 一致的可执行动作列表 */
export function resolveProjectActions(project) {
  if (!project) return [];
  if (project.kind === "maven") {
    const actions = ["clean", "install", "clean install", "package"];
    if (project.springBoot) actions.push("spring-boot:run");
    return actions;
  }
  if (project.kind === "cargo" || project.kind === "gradle") {
    return [...(project.scripts || [])];
  }
  return ["install", ...(project.scripts || []).map((name) => `script:${name}`)];
}

/**
 * 勾选命令为过滤条件：过滤掉「这些命令全都不具备」的项目
 * （即：具备任一勾选命令则显示）。未勾选任何命令 → 不按命令过滤。
 * Maven onlyJar：非 jar packaging 一律过滤。
 * @param {object} project
 * @param {ProjectFilter|null|undefined} filter
 */
export function projectPassesFilter(project, filter) {
  if (!project) return false;
  const kind = String(project.kind || "").toLowerCase();
  const rule = filter?.[kind];
  const required = Array.isArray(rule?.actions) ? rule.actions : [];
  if (required.length) {
    const have = new Set(resolveProjectActions(project));
    if (!required.some((action) => have.has(action))) return false;
  }
  if (kind === "maven" && rule?.onlyJar) {
    const packaging = String(project.packaging || "jar").toLowerCase();
    if (packaging !== "jar") return false;
  }
  return true;
}

/**
 * @param {object[]} projects
 * @param {ProjectFilter|null|undefined} filter
 */
export function filterProjects(projects, filter) {
  const list = Array.isArray(projects) ? projects : [];
  return list.filter((p) => projectPassesFilter(p, filter));
}

/**
 * @param {object} settings
 * @param {string} kind
 * @param {string[]} actions
 */
export function setKindFilterActions(settings, kind, actions) {
  const projectFilter = normalizeProjectFilter(settings?.projectFilter);
  const k = String(kind || "").toLowerCase();
  if (!FILTER_KINDS.includes(k)) return settings;
  const allowed = new Set(allActionValues(k));
  const prev = projectFilter[k] || defaultKindRule(k);
  projectFilter[k] = {
    ...prev,
    actions: (Array.isArray(actions) ? actions : []).filter((a) => allowed.has(a)),
  };
  return { ...settings, projectFilter };
}

/**
 * @param {object} settings
 * @param {string} kind
 * @param {string} action
 * @param {boolean} checked
 */
export function toggleKindFilterAction(settings, kind, action, checked) {
  const projectFilter = normalizeProjectFilter(settings?.projectFilter);
  const k = String(kind || "").toLowerCase();
  if (!FILTER_KINDS.includes(k)) return settings;
  const allowed = new Set(allActionValues(k));
  if (!allowed.has(action)) return settings;
  const prev = projectFilter[k] || defaultKindRule(k);
  const next = new Set(prev.actions || []);
  if (checked) next.add(action);
  else next.delete(action);
  projectFilter[k] = { ...prev, actions: [...next] };
  return { ...settings, projectFilter };
}

/**
 * @param {object} settings
 * @param {boolean} onlyJar
 */
export function setMavenOnlyJar(settings, onlyJar) {
  const projectFilter = normalizeProjectFilter(settings?.projectFilter);
  const prev = projectFilter.maven || defaultKindRule("maven");
  projectFilter.maven = { ...prev, onlyJar: !!onlyJar };
  return { ...settings, projectFilter };
}

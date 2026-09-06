import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import type { CatalogModel, ModelsCatalog } from "../src/lib/types";

const CATALOG_REL = "config/models.json";

const CATALOG_NOTE =
  "Slug catalog only. No secrets and no system/developer prompts. Desk injects briefs/OPERATOR.md via exec --config. Set config.toml model= to a catalog slug.";

const FORBIDDEN = new Set([
  "system",
  "systemprompt",
  "systeminstructions",
  "developerinstructions",
  "developerprompt",
  "modelinstructions",
  "modelinstructionsfile",
  "instructions",
  "prompt",
  "prompts",
  "messages",
  "apikey",
  "pat",
  "token",
  "secret",
  "bearer",
  "baseurl",
  "endpoint",
  "envkey",
  "wireapi",
]);

function normalizeKey(key: string): string {
  return key.replace(/[_-]/g, "").toLowerCase();
}

function collectForbidden(value: unknown, out: string[]): void {
  if (Array.isArray(value)) {
    for (const item of value) collectForbidden(item, out);
    return;
  }
  if (value && typeof value === "object") {
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
      if (FORBIDDEN.has(normalizeKey(k))) out.push(k);
      collectForbidden(v, out);
    }
  }
}

function slugFromObject(obj: Record<string, unknown>): CatalogModel | null {
  const raw = obj.slug ?? obj.id;
  const slug = typeof raw === "string" ? raw.trim() : "";
  if (!slug) return null;
  const labelRaw = obj.label ?? obj.name ?? obj.display_name;
  const providerRaw = obj.provider;
  return {
    slug,
    label: typeof labelRaw === "string" && labelRaw.trim() ? labelRaw.trim() : null,
    provider: typeof providerRaw === "string" && providerRaw.trim() ? providerRaw.trim() : null,
  };
}

function pushEntries(items: unknown[], out: CatalogModel[]): void {
  for (const item of items) {
    if (typeof item === "string" && item.trim()) {
      out.push({ slug: item.trim(), label: null, provider: null });
    } else if (item && typeof item === "object") {
      const model = slugFromObject(item as Record<string, unknown>);
      if (model) out.push(model);
    }
  }
}

export function parseCatalog(text: string): CatalogModel[] {
  const value = JSON.parse(text) as unknown;
  const forbidden: string[] = [];
  collectForbidden(value, forbidden);
  const unique = [...new Set(forbidden)].sort();
  if (unique.length) {
    throw new Error(
      `models.json must be slugs/catalog only. Remove prompt/secret/endpoint keys: ${unique.join(", ")}`,
    );
  }
  const models: CatalogModel[] = [];
  if (Array.isArray(value)) {
    pushEntries(value, models);
  } else if (value && typeof value === "object") {
    const root = value as Record<string, unknown>;
    if (Array.isArray(root.models) || Array.isArray(root.slugs)) {
      pushEntries((root.models as unknown[]) || (root.slugs as unknown[]), models);
    } else if (root.models && typeof root.models === "object") {
      for (const [slug, entry] of Object.entries(root.models as Record<string, unknown>)) {
        const trimmed = slug.trim();
        if (!trimmed) continue;
        if (entry && typeof entry === "object") {
          const model = slugFromObject(entry as Record<string, unknown>);
          if (model) {
            models.push({ ...model, slug: trimmed });
            continue;
          }
        }
        models.push({ slug: trimmed, label: null, provider: null });
      }
    }
  }
  const seen = new Set<string>();
  const deduped = models.filter((m) => {
    if (seen.has(m.slug)) return false;
    seen.add(m.slug);
    return true;
  });
  if (!deduped.length) {
    throw new Error("models.json has no slugs. Add models[].slug (or a slugs array).");
  }
  return deduped;
}

export function loadCatalog(cwd = process.cwd()): ModelsCatalog {
  const filePath = path.join(cwd, "config", "models.json");
  if (!existsSync(filePath)) {
    return {
      path: filePath,
      exists: false,
      ok: false,
      error: `No ${CATALOG_REL} in the workspace. Add a slug catalog (no prompts, no secrets).`,
      slugs: [],
      models: [],
      note: CATALOG_NOTE,
    };
  }
  try {
    const models = parseCatalog(readFileSync(filePath, "utf8"));
    return {
      path: filePath,
      exists: true,
      ok: true,
      error: null,
      slugs: models.map((m) => m.slug),
      models,
      note: CATALOG_NOTE,
    };
  } catch (err) {
    return {
      path: filePath,
      exists: true,
      ok: false,
      error: err instanceof Error ? err.message : String(err),
      slugs: [],
      models: [],
      note: CATALOG_NOTE,
    };
  }
}

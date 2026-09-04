import { appendFileSync, existsSync, mkdirSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import path from "node:path";
import { loadOrCreateDek, looksLikeEnvelope, open, seal, sessionUser } from "./crypto";
import type { AuditEvent } from "../src/lib/types";

export const DATA_DIR = path.resolve(process.cwd(), ".data");
const ENC_PATH = path.join(DATA_DIR, "preview-store.json.enc");
const LEGACY = path.join(DATA_DIR, "preview-store.json");
const LEGACY_AUDIT = path.join(DATA_DIR, "audit.jsonl");
const GENESIS = "0".repeat(64);

export type StoreFile = Record<string, unknown>;

function dek() {
  return loadOrCreateDek(DATA_DIR);
}

export function loadStore(): StoreFile {
  mkdirSync(DATA_DIR, { recursive: true, mode: 0o700 });
  if (existsSync(ENC_PATH)) {
    const { dek: key } = dek();
    const raw = open(key, readFileSync(ENC_PATH)).toString("utf8");
    return JSON.parse(raw) as StoreFile;
  }
  if (existsSync(LEGACY)) {
    const parsed = JSON.parse(readFileSync(LEGACY, "utf8")) as StoreFile;
    saveStore(parsed);
    overwriteRemove(LEGACY);
    if (existsSync(LEGACY_AUDIT)) overwriteRemove(LEGACY_AUDIT);
    return parsed;
  }
  return {};
}

export function saveStore(store: StoreFile) {
  mkdirSync(DATA_DIR, { recursive: true, mode: 0o700 });
  const { dek: key } = dek();
  const blob = seal(key, Buffer.from(JSON.stringify(store), "utf8"));
  writeFileSync(ENC_PATH, blob, { mode: 0o600 });
  if (existsSync(LEGACY)) overwriteRemove(LEGACY);
}

function overwriteRemove(filePath: string) {
  try {
    const size = readFileSync(filePath).length;
    writeFileSync(filePath, Buffer.alloc(size));
    unlinkSync(filePath);
  } catch {
    // ignore
  }
}

export function patchStore(patch: StoreFile): StoreFile {
  const next = { ...loadStore(), ...patch };
  saveStore(next);
  return next;
}

function lastHash(events: AuditEvent[]): string {
  const chained = [...events].reverse().find((e) => e.event_hash);
  return chained?.event_hash || GENESIS;
}

export function eventHash(prev: string, event: Omit<AuditEvent, "prev_hash" | "event_hash">): string {
  return createHash("sha256")
    .update(prev)
    .update("\0")
    .update(event.id)
    .update("\0")
    .update(event.at)
    .update("\0")
    .update(event.action)
    .update("\0")
    .update(event.actor)
    .update("\0")
    .update(event.entity_type)
    .update("\0")
    .update(event.entity_id)
    .update("\0")
    .update(event.detail)
    .digest("hex");
}

export function redactDetail(detail: string): string {
  return detail
    .split(/\r?\n/)
    .map((line) => {
      const upper = line.toUpperCase();
      if (
        (upper.includes("PAT") ||
          upper.includes("API_KEY") ||
          upper.includes("TOKEN") ||
          upper.includes("SECRET") ||
          upper.includes("BEARER ")) &&
        (line.includes("=") || line.includes(":"))
      ) {
        return "[redacted]";
      }
      return line;
    })
    .join("\n");
}

export function writeAudit(action: string, entityType: string, entityId: string, detail: string): AuditEvent {
  const store = loadStore();
  const list = ((store.audit as AuditEvent[] | undefined) ?? []).slice();
  const prev = lastHash(list);
  const base = {
    id: randomUUID(),
    at: new Date().toISOString(),
    action,
    actor: `local-user:${sessionUser()}`,
    entity_type: entityType,
    entity_id: entityId,
    detail: redactDetail(detail),
  };
  const event: AuditEvent = {
    ...base,
    prev_hash: prev,
    event_hash: eventHash(prev, base),
  };
  list.unshift(event);
  patchStore({ audit: list.slice(0, 400) });
  return event;
}

export function auditChainOk(): boolean {
  const events = ([...((loadStore().audit as AuditEvent[] | undefined) ?? [])] as AuditEvent[]).reverse();
  let prev = GENESIS;
  for (const event of events) {
    if (!event.event_hash || event.prev_hash !== prev) return false;
    const expected = eventHash(prev, event);
    if (expected !== event.event_hash) return false;
    prev = event.event_hash;
  }
  return true;
}

export function writeUnlockFailure(reason: string) {
  mkdirSync(DATA_DIR, { recursive: true, mode: 0o700 });
  appendFileSync(
    path.join(DATA_DIR, "unlock-failures.jsonl"),
    `${JSON.stringify({
      at: new Date().toISOString(),
      action: "encryption.key_unlock_failure",
      detail: "key unlock failed (no key material logged)",
      code: reason.includes("tamper") ? "tamper_or_wrong_key" : "unlock_failed",
    })}\n`,
  );
}

export function leftoverPlaintext(): boolean {
  return existsSync(LEGACY) || existsSync(LEGACY_AUDIT);
}

export function storeEncryptedOnDisk(): boolean {
  if (!existsSync(ENC_PATH)) return false;
  try {
    return looksLikeEnvelope(readFileSync(ENC_PATH));
  } catch {
    return false;
  }
}

import { createCipheriv, createDecipheriv, createHash, hkdfSync, randomBytes } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { hostname, userInfo } from "node:os";
import path from "node:path";

export const MAGIC = Buffer.from("CDEX1");
const VERSION = 1;
const NONCE_LEN = 12;

export function machineBinding(): string {
  const mid = machineId();
  const user = sessionUser();
  return createHash("sha256").update("codex-desk-bind-v1").update(mid).update("\0").update(user).digest("hex");
}

export function sessionUser(): string {
  return process.env.USERNAME || process.env.USER || userInfo().username || "unknown-user";
}

export function machineId(): string {
  for (const candidate of ["/etc/machine-id", "/var/lib/dbus/machine-id"]) {
    if (existsSync(candidate)) {
      const id = readFileSync(candidate, "utf8").trim();
      if (id) return id;
    }
  }
  return process.env.COMPUTERNAME || process.env.HOSTNAME || hostname();
}

export function helloBind(): string {
  return process.platform === "win32" ? "windows-user-session" : "posix-user-session";
}

function machineKek(): Buffer {
  const ikm = `${machineId()}|${sessionUser()}`;
  return Buffer.from(hkdfSync("sha256", ikm, "codex-desk-il5-store-v1", "dek-wrap", 32));
}

export function seal(key: Buffer, plaintext: Buffer): Buffer {
  const nonce = randomBytes(NONCE_LEN);
  const cipher = createCipheriv("aes-256-gcm", key, nonce);
  const ct = Buffer.concat([cipher.update(plaintext), cipher.final()]);
  const tag = cipher.getAuthTag();
  return Buffer.concat([MAGIC, Buffer.from([VERSION]), nonce, ct, tag]);
}

export function open(key: Buffer, blob: Buffer): Buffer {
  if (blob.length < MAGIC.length + 1 + NONCE_LEN + 16) {
    throw new Error("encrypted store is truncated");
  }
  if (!blob.subarray(0, 5).equals(MAGIC)) {
    throw new Error("not a Codex Desk encrypted store (missing CDEX1 magic)");
  }
  if (blob[5] !== VERSION) {
    throw new Error(`unsupported store version ${blob[5]}`);
  }
  const nonce = blob.subarray(6, 18);
  const tag = blob.subarray(blob.length - 16);
  const ct = blob.subarray(18, blob.length - 16);
  const decipher = createDecipheriv("aes-256-gcm", key, nonce);
  decipher.setAuthTag(tag);
  return Buffer.concat([decipher.update(ct), decipher.final()]);
}

export function looksLikeEnvelope(buf: Buffer): boolean {
  return buf.length >= 5 + 1 + NONCE_LEN + 16 && buf.subarray(0, 5).equals(MAGIC);
}

export function loadOrCreateDek(appData: string): { dek: Buffer; backend: string } {
  mkdirSync(appData, { recursive: true, mode: 0o700 });
  const wrapPath = path.join(appData, "dek.wrap");
  const kek = machineKek();
  if (existsSync(wrapPath)) {
    const dek = open(kek, readFileSync(wrapPath));
    if (dek.length !== 32) throw new Error("dek is not 32 bytes");
    return { dek, backend: "machine-bound" };
  }
  const dek = randomBytes(32);
  writeFileSync(wrapPath, seal(kek, dek), { mode: 0o600 });
  return { dek, backend: "machine-bound" };
}

export function setPatSlot(appData: string, pat: string): string {
  const trimmed = pat.trim();
  if (!trimmed) throw new Error("PAT is empty.");
  const { dek } = loadOrCreateDek(appData);
  writeFileSync(path.join(appData, "pat.wrap"), seal(dek, Buffer.from(trimmed, "utf8")), { mode: 0o600 });
  return "os-secret-store";
}

export function getPatSlot(appData: string): string | undefined {
  const wrap = path.join(appData, "pat.wrap");
  if (!existsSync(wrap)) return undefined;
  const { dek } = loadOrCreateDek(appData);
  const value = open(dek, readFileSync(wrap)).toString("utf8");
  return value || undefined;
}

export function clearPatSlot(appData: string) {
  const wrap = path.join(appData, "pat.wrap");
  if (existsSync(wrap)) {
    writeFileSync(wrap, Buffer.alloc(32));
    unlinkSync(wrap);
  }
}

export function patSlotPresent(appData: string): boolean {
  return existsSync(path.join(appData, "pat.wrap"));
}

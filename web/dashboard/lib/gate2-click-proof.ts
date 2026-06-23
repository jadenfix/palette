export const GATE2_CLICK_PROOF_NONCE = /^[0-9a-f]{32}$/;

export type BrowserClickProof = {
  nonce: string;
  capturedAtMs: number;
  isTrusted: true;
  button: number;
  detail: number;
  clientX: number;
  clientY: number;
  screenX: number;
  screenY: number;
};

export function isBrowserClickProof(value: unknown): value is BrowserClickProof {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const record = value as Record<string, unknown>;
  return (
    typeof record.nonce === "string" &&
    GATE2_CLICK_PROOF_NONCE.test(record.nonce) &&
    Number.isInteger(record.capturedAtMs) &&
    record.isTrusted === true &&
    record.button === 0 &&
    typeof record.detail === "number" &&
    Number.isInteger(record.detail) &&
    record.detail >= 1 &&
    finiteNumber(record.clientX) &&
    finiteNumber(record.clientY) &&
    finiteNumber(record.screenX) &&
    finiteNumber(record.screenY)
  );
}

function finiteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

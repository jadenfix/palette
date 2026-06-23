import { createHash } from "node:crypto";

import {
  GATE2_CONFIRMATION_HASH_PREFIX,
  GATE2_CONFIRMATION_SALT_ENV
} from "./gate2-confirmation-contract";

export {
  GATE2_CONFIRMATION_CODE,
  GATE2_CONFIRMATION_HASH_PREFIX,
  GATE2_CONFIRMATION_SALT_ENV
} from "./gate2-confirmation-contract";

export function gate2ConfirmationSalt(env: NodeJS.ProcessEnv = process.env): string {
  return env[GATE2_CONFIRMATION_SALT_ENV] ?? "";
}

export function gate2ConfirmationCode({
  salt = gate2ConfirmationSalt(),
  traceId,
  spanId
}: {
  salt?: string;
  traceId: string;
  spanId: string;
}): string {
  return createHash("sha256")
    .update(`${GATE2_CONFIRMATION_HASH_PREFIX}:${salt}:${traceId}:${spanId}`)
    .digest("hex")
    .slice(0, 8)
    .toUpperCase();
}

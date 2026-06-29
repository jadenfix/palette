/** Optional provider auto-instrumentation for installed LLM SDKs. */

import { wrapAnthropic } from "./providers/anthropic";
import { wrapOpenAI } from "./providers/openai";

/* eslint-disable @typescript-eslint/no-explicit-any */

export type InstrumentProvider = "openai" | "anthropic";

export interface InstrumentOptions {
  /** Providers to patch. Defaults to OpenAI and Anthropic. */
  providers?: InstrumentProvider[];
  /**
   * Optional module overrides, primarily for tests or custom loaders. A null
   * value marks that provider as missing instead of trying runtime require().
   */
  modules?: Partial<Record<InstrumentProvider, unknown | null>>;
}

export interface SkippedProvider {
  provider: InstrumentProvider;
  reason: string;
}

export interface InstrumentResult {
  providers: InstrumentProvider[];
  skipped: SkippedProvider[];
}

type RequireLike = {
  (id: string): unknown;
  resolve?: (id: string) => string;
  cache?: Record<string, { exports: unknown } | undefined>;
};

declare const require: RequireLike | undefined;

type LoadedModule = {
  value: unknown;
  replace?: (value: unknown) => void;
};

type Constructor = new (...args: any[]) => any;
type ClientWrapper = <T extends Record<string, any>>(client: T) => T;

const DEFAULT_PROVIDERS: InstrumentProvider[] = ["openai", "anthropic"];
const CONSTRUCTOR_MARK = Symbol.for("@beater/sdk.auto.constructor");

const MODULE_IDS: Record<InstrumentProvider, string[]> = {
  openai: ["openai"],
  anthropic: ["@anthropic-ai/sdk", "anthropic"],
};

const CONSTRUCTOR_EXPORTS: Record<InstrumentProvider, string[]> = {
  openai: ["default", "OpenAI"],
  anthropic: ["default", "Anthropic"],
};

const WRAPPERS: Record<InstrumentProvider, ClientWrapper> = {
  openai: wrapOpenAI,
  anthropic: wrapAnthropic,
};

function getRequire(): RequireLike | undefined {
  try {
    return typeof require === "function" ? require : undefined;
  } catch {
    return undefined;
  }
}

function isMissingModule(err: any, id: string): boolean {
  if (err?.code !== "MODULE_NOT_FOUND" && err?.code !== "ERR_MODULE_NOT_FOUND") return false;
  const message = String(err?.message ?? "");
  return message.includes(`'${id}'`) || message.includes(`"${id}"`) || message.includes(id);
}

function loadOptionalModule(provider: InstrumentProvider): LoadedModule | undefined {
  const runtimeRequire = getRequire();
  if (!runtimeRequire) return undefined;

  for (const id of MODULE_IDS[provider]) {
    try {
      const value = runtimeRequire(id);
      const resolved = runtimeRequire.resolve?.(id);
      const entry = resolved ? runtimeRequire.cache?.[resolved] : undefined;
      return {
        value,
        replace: entry ? (next) => {
          entry.exports = next;
        } : undefined,
      };
    } catch (err) {
      if (isMissingModule(err, id)) continue;
      throw err;
    }
  }

  return undefined;
}

function loadProviderModule(provider: InstrumentProvider, options: InstrumentOptions): LoadedModule | undefined {
  if (options.modules && Object.prototype.hasOwnProperty.call(options.modules, provider)) {
    const value = options.modules[provider];
    return value == null ? undefined : { value };
  }
  return loadOptionalModule(provider);
}

function wrapConstructor(original: Constructor, wrapClient: ClientWrapper): Constructor {
  const existing = (original as any)[CONSTRUCTOR_MARK];
  if (existing) return existing;

  const wrapped = function BeaterInstrumentedProvider(this: unknown, ...args: any[]) {
    const instance = Reflect.construct(original, args, new.target ?? wrapped);
    return wrapClient(instance);
  } as unknown as Constructor;

  Object.setPrototypeOf(wrapped, original);
  wrapped.prototype = original.prototype;
  (original as any)[CONSTRUCTOR_MARK] = wrapped;
  (wrapped as any)[CONSTRUCTOR_MARK] = wrapped;
  return wrapped;
}

function setExport(target: any, key: string, value: unknown): boolean {
  try {
    target[key] = value;
    return target[key] === value;
  } catch {
    return false;
  }
}

function patchProviderExports(provider: InstrumentProvider, loaded: LoadedModule): boolean {
  const wrapClient = WRAPPERS[provider];
  const replacements = new Map<Constructor, Constructor>();

  const replacementFor = (candidate: unknown): Constructor | undefined => {
    if (typeof candidate !== "function") return undefined;
    const original = candidate as Constructor;
    const existing = replacements.get(original);
    if (existing) return existing;
    const wrapped = wrapConstructor(original, wrapClient);
    replacements.set(original, wrapped);
    return wrapped;
  };

  const directReplacement = replacementFor(loaded.value);
  if (directReplacement && loaded.replace) {
    loaded.replace(directReplacement);
    return true;
  }

  if (!loaded.value || (typeof loaded.value !== "object" && typeof loaded.value !== "function")) {
    return false;
  }

  let patched = false;
  const exportsObject = loaded.value as Record<string, unknown>;
  for (const key of CONSTRUCTOR_EXPORTS[provider]) {
    const replacement = replacementFor(exportsObject[key]);
    if (replacement) {
      patched = setExport(exportsObject, key, replacement) || patched;
    }
  }

  return patched;
}

/**
 * Patch installed provider SDK constructors so clients created after this call
 * are wrapped with Beater spans. Missing optional dependencies are skipped.
 */
export function instrument(options: InstrumentOptions = {}): InstrumentResult {
  const providers = options.providers ?? DEFAULT_PROVIDERS;
  const result: InstrumentResult = { providers: [], skipped: [] };

  for (const provider of providers) {
    const loaded = loadProviderModule(provider, options);
    if (!loaded) {
      result.skipped.push({ provider, reason: "provider module is not installed" });
      continue;
    }

    if (patchProviderExports(provider, loaded)) {
      result.providers.push(provider);
    } else {
      result.skipped.push({ provider, reason: "provider module shape was not recognized" });
    }
  }

  return result;
}

export const auto = { instrument };

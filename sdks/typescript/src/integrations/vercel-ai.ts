/** Helpers for enabling Vercel AI SDK telemetry without adding an `ai` dependency. */

export interface VercelAiTelemetryConfig {
  isEnabled: boolean;
  functionId?: string;
  recordInputs?: boolean;
  recordOutputs?: boolean;
  metadata?: Record<string, unknown>;
}

export interface BeaterVercelAiTelemetryOptions {
  enabled?: boolean;
  functionId?: string;
  recordInputs?: boolean;
  recordOutputs?: boolean;
  metadata?: Record<string, unknown>;
}

export type VercelAiTelemetryOptionName = "telemetry" | "experimental_telemetry";

export interface WithVercelAiTelemetryOptions extends BeaterVercelAiTelemetryOptions {
  optionName?: VercelAiTelemetryOptionName;
}

export function vercelAiTelemetry(options: BeaterVercelAiTelemetryOptions = {}): VercelAiTelemetryConfig {
  const config: VercelAiTelemetryConfig = {
    isEnabled: options.enabled ?? true,
  };

  if (options.functionId !== undefined) {
    config.functionId = options.functionId;
  }
  if (options.recordInputs !== undefined) {
    config.recordInputs = options.recordInputs;
  }
  if (options.recordOutputs !== undefined) {
    config.recordOutputs = options.recordOutputs;
  }
  if (options.metadata !== undefined) {
    config.metadata = { ...options.metadata };
  }

  return config;
}

export function withVercelAiTelemetry<T extends Record<string, unknown>>(
  request: T,
  options: WithVercelAiTelemetryOptions = {},
): T & Partial<Record<VercelAiTelemetryOptionName, VercelAiTelemetryConfig>> {
  const { optionName = "experimental_telemetry", ...telemetryOptions } = options;

  return {
    ...request,
    [optionName]: vercelAiTelemetry(telemetryOptions),
  };
}

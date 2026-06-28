/**
 * Beater brand mark — a "beat" pulse inside a rounded ink tile. Used in the nav,
 * auth screen, and account menu so the identity is consistent everywhere.
 */
export function BeaterMark({ size = 28 }: { size?: number }) {
  const radius = Math.round(size * 0.28);
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      role="img"
      aria-label="Beater"
      style={{ display: "block", flex: "0 0 auto" }}
    >
      <rect width="32" height="32" rx={radius} fill="#14171a" />
      <path
        d="M5 17.5h4.4l2.1-6.5a1 1 0 0 1 1.9.05l3.4 11.4a1 1 0 0 0 1.9.06L22 15h5"
        fill="none"
        stroke="#2fd4c4"
        strokeWidth="2.1"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

/** Brand mark + wordmark lockup. */
export function BrandLockup({ size = 28 }: { size?: number }) {
  return (
    <span className="brand-lockup">
      <BeaterMark size={size} />
      <b>Beater</b>
    </span>
  );
}

/**
 * Live agent-vitals readout — the brand panel's signature element. One EKG
 * complex (P-QRS-T) repeated across the strip, with a bright pulse that sweeps
 * left-to-right like a patient monitor. `pathLength={100}` normalises the dash
 * so the sweep is resolution-independent; `vector-effect` keeps the stroke even
 * under the non-uniform horizontal stretch. Decorative, so aria-hidden.
 */
export function BrandVitals() {
  // One 150-unit P-QRS-T complex, repeated four times across the 600-wide field.
  const complex = "h46 l5 -6 l5 6 h24 l6 22 l7 -46 l7 40 l6 -16 h44";
  const d = `M0 40 ${complex} ${complex} ${complex} ${complex}`;
  return (
    <svg
      className="ekg"
      viewBox="0 0 600 80"
      preserveAspectRatio="none"
      aria-hidden="true"
      focusable="false"
    >
      <path className="ekg-base" d={d} pathLength={100} vectorEffect="non-scaling-stroke" />
      <path className="ekg-scan" d={d} pathLength={100} vectorEffect="non-scaling-stroke" />
    </svg>
  );
}

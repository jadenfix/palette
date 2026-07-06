#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bare-metal-optimize-env.sh [--emit-env] [--emit-json]

Outputs bare-metal tuning recommendations for host-local CI workloads.

Flags:
  --emit-env   Output bash export statements for recommended variables.
  --emit-json  Output compact JSON only (for machine consumption).
EOF
}

emit_env=0
emit_json=0

for arg in "$@"; do
  case "$arg" in
    --emit-env)
      emit_env=1
      ;;
    --emit-json)
      emit_json=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      usage
      exit 1
      ;;
  esac
done

if ! command -v nproc >/dev/null 2>&1; then
  logical_cpu=1
else
  logical_cpu="$(nproc --all)"
fi

physical_cpu="$logical_cpu"
if [[ -f /proc/cpuinfo ]]; then
  physical_guess="$(awk '/^physical id:/{print $4}' /proc/cpuinfo | sort -un | wc -l | tr -d ' ')"
  if [[ "$physical_guess" =~ ^[0-9]+$ ]] && [[ "$physical_guess" -gt 0 ]]; then
    physical_cpu="$physical_guess"
  fi
fi

if [[ ! "$logical_cpu" =~ ^[0-9]+$ ]] || [[ "$logical_cpu" -le 0 ]]; then
  logical_cpu=1
fi
if [[ ! "$physical_cpu" =~ ^[0-9]+$ ]] || [[ "$physical_cpu" -le 0 ]]; then
  physical_cpu=1
fi

if [[ -f /proc/meminfo ]]; then
  total_mem_kib="$(awk '/MemTotal:/ {print $2; exit}' /proc/meminfo | tr -dc '0-9')"
else
  total_mem_kib=0
fi

if [[ "$total_mem_kib" =~ ^[0-9]+$ ]] && [[ "$total_mem_kib" -gt 0 ]]; then
  total_mem_gib="$(awk -v mib="$total_mem_kib" 'BEGIN { printf "%.2f", mib / 1024 / 1024 }')"
else
  total_mem_gib=0
fi

has_avx2=0
if grep -Eq '^flags\s*:.*\bavx2\b' /proc/cpuinfo 2>/dev/null; then
  has_avx2=1
fi

has_cuda=0
if command -v nvidia-smi >/dev/null 2>&1; then
  if nvidia-smi >/dev/null 2>&1; then
    has_cuda=1
  fi
fi

if (( logical_cpu >= 4 )); then
  cargo_jobs=$((logical_cpu - 1))
else
  cargo_jobs=1
fi

if (( physical_cpu >= 8 )); then
  rust_flags='-C target-cpu=native -C codegen-units=1 -C opt-level=3'
else
  rust_flags='-C target-cpu=native -C codegen-units=1 -C opt-level=2'
fi

if [[ "$total_mem_gib" != "0" ]]; then
  if (( $(awk -v m="$total_mem_gib" 'BEGIN { print (m < 12) }') == 1 )); then
    linker_jobs=1
  else
    linker_jobs=$((cargo_jobs > 4 ? 4 : cargo_jobs))
  fi
else
  linker_jobs=2
fi

if [[ "${BM_CARGO_BUILD_JOBS-}" == "" ]]; then
  BM_CARGO_BUILD_JOBS_DEFAULT="$cargo_jobs"
else
  BM_CARGO_BUILD_JOBS_DEFAULT="$BM_CARGO_BUILD_JOBS"
fi

if [[ "${BM_RUSTFLAGS-}" == "" ]]; then
  BM_RUSTFLAGS_DEFAULT="$rust_flags"
else
  BM_RUSTFLAGS_DEFAULT="$BM_RUSTFLAGS"
fi

if [[ "${BM_LINKER_JOBS-}" == "" ]]; then
  BM_LINKER_JOBS_DEFAULT="$linker_jobs"
else
  BM_LINKER_JOBS_DEFAULT="$BM_LINKER_JOBS"
fi

if (( logical_cpu >= 2 )); then
  go_parallel=$((logical_cpu))
else
  go_parallel=1
fi
if [[ "${BM_GOMAXPROCS-}" == "" ]]; then
  BM_GOMAXPROCS_DEFAULT="$go_parallel"
else
  BM_GOMAXPROCS_DEFAULT="$BM_GOMAXPROCS"
fi

if command -v go >/dev/null 2>&1 && [[ "${BM_GOFLAGS-}" == "" ]]; then
  BM_GOFLAGS_DEFAULT="-p=$go_parallel"
else
  BM_GOFLAGS_DEFAULT="${BM_GOFLAGS:-}"
fi

if command -v mvn >/dev/null 2>&1 && [[ "${BM_MAVEN_OPTS-}" == "" ]]; then
  BM_MAVEN_OPTS_DEFAULT="-Xmx2g -XX:MaxMetaspaceSize=512m -Djava.awt.headless=true"
else
  BM_MAVEN_OPTS_DEFAULT="${BM_MAVEN_OPTS:-}"
fi

if command -v dotnet >/dev/null 2>&1 && [[ "${BM_DOTNET_MAX_CPU_COUNT-}" == "" ]]; then
  BM_DOTNET_MAX_CPU_COUNT_DEFAULT="$logical_cpu"
else
  BM_DOTNET_MAX_CPU_COUNT_DEFAULT="${BM_DOTNET_MAX_CPU_COUNT:-}"
fi

if command -v make >/dev/null 2>&1 && [[ "${BM_MAKEFLAGS-}" == "" ]]; then
  BM_MAKEFLAGS_DEFAULT="-j$logical_cpu"
else
  BM_MAKEFLAGS_DEFAULT="${BM_MAKEFLAGS:-}"
fi

if [[ "${BM_CMAKE_BUILD_PARALLEL_LEVEL-}" == "" ]]; then
  BM_CMAKE_BUILD_PARALLEL_LEVEL_DEFAULT="$logical_cpu"
else
  BM_CMAKE_BUILD_PARALLEL_LEVEL_DEFAULT="$BM_CMAKE_BUILD_PARALLEL_LEVEL"
fi

if (( emit_json == 1 )); then
  cat <<JSON
{
  "logical_cpu": $logical_cpu,
  "physical_cpu": $physical_cpu,
  "memory_gib": $total_mem_gib,
  "has_avx2": $has_avx2,
  "has_cuda": $has_cuda,
  "recommendation": {
    "CARGO_BUILD_JOBS": "$BM_CARGO_BUILD_JOBS_DEFAULT",
    "CARGO_BUILD_JOBS_LINKER": "$BM_LINKER_JOBS_DEFAULT",
    "RUSTFLAGS": "$BM_RUSTFLAGS_DEFAULT",
    "GOMAXPROCS": "$BM_GOMAXPROCS_DEFAULT",
    "GOFLAGS": "$BM_GOFLAGS_DEFAULT",
    "MAVEN_OPTS": "$BM_MAVEN_OPTS_DEFAULT",
    "DOTNET_MAX_CPU_COUNT": "$BM_DOTNET_MAX_CPU_COUNT_DEFAULT",
    "MAKEFLAGS": "$BM_MAKEFLAGS_DEFAULT",
    "CMAKE_BUILD_PARALLEL_LEVEL": "$BM_CMAKE_BUILD_PARALLEL_LEVEL_DEFAULT"
  }
}
JSON
  exit 0
fi

if (( emit_env == 1 )); then
  printf 'export BEATER_BM_OPT_PROFILE=auto\n'
  printf 'export BEATER_BM_OPT_LOGICAL_CPU=%q\n' "$logical_cpu"
  printf 'export BEATER_BM_OPT_PHYSICAL_CPU=%q\n' "$physical_cpu"
  printf 'export BEATER_BM_OPT_MEMORY_GIB=%q\n' "$total_mem_gib"
  printf 'export BEATER_BM_OPT_HAS_AVX2=%q\n' "$has_avx2"
  printf 'export BEATER_BM_OPT_HAS_CUDA=%q\n' "$has_cuda"
  printf 'export CARGO_BUILD_JOBS=%q\n' "$BM_CARGO_BUILD_JOBS_DEFAULT"
  printf 'export CARGO_BUILD_JOBS_LINKER=%q\n' "$BM_LINKER_JOBS_DEFAULT"
  printf 'export CMAKE_BUILD_PARALLEL_LEVEL=%q\n' "$BM_CMAKE_BUILD_PARALLEL_LEVEL_DEFAULT"
  printf 'export RUSTFLAGS=%q\n' "$BM_RUSTFLAGS_DEFAULT"
  printf 'export GOMAXPROCS=%q\n' "$BM_GOMAXPROCS_DEFAULT"
  printf 'export GOFLAGS=%q\n' "$BM_GOFLAGS_DEFAULT"
  if [[ -n "$BM_MAVEN_OPTS_DEFAULT" ]]; then
    printf 'export MAVEN_OPTS=%q\n' "$BM_MAVEN_OPTS_DEFAULT"
  fi
  if [[ -n "$BM_DOTNET_MAX_CPU_COUNT_DEFAULT" ]]; then
    printf 'export DOTNET_MAX_CPU_COUNT=%q\n' "$BM_DOTNET_MAX_CPU_COUNT_DEFAULT"
  fi
  if [[ -n "$BM_MAKEFLAGS_DEFAULT" ]]; then
    printf 'export MAKEFLAGS=%q\n' "$BM_MAKEFLAGS_DEFAULT"
  fi
  exit 0
fi

cat <<EOF
BEATER_BM_OPT_PROFILE=auto
Logical CPU: $logical_cpu
Physical CPU: $physical_cpu
Memory: ${total_mem_gib} GiB
Has AVX2: $has_avx2
Has CUDA: $has_cuda
Recommended:
  CARGO_BUILD_JOBS=$BM_CARGO_BUILD_JOBS_DEFAULT
  RUSTFLAGS=$BM_RUSTFLAGS_DEFAULT
  CARGO_BUILD_JOBS_LINKER=$BM_LINKER_JOBS_DEFAULT
  GOMAXPROCS=$BM_GOMAXPROCS_DEFAULT
  GOFLAGS=${BM_GOFLAGS_DEFAULT}
  MAVEN_OPTS=${BM_MAVEN_OPTS_DEFAULT}
  DOTNET_MAX_CPU_COUNT=${BM_DOTNET_MAX_CPU_COUNT_DEFAULT}
  MAKEFLAGS=${BM_MAKEFLAGS_DEFAULT}
  CMAKE_BUILD_PARALLEL_LEVEL=${BM_CMAKE_BUILD_PARALLEL_LEVEL_DEFAULT}
EOF

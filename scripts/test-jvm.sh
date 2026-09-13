#!/usr/bin/env bash
set -euo pipefail

if (( $# > 1 )); then
    echo "Usage: bash scripts/test-jvm.sh [path-to-kotlinc]" >&2
    exit 2
fi
if ! command -v python3 >/dev/null 2>&1; then
    echo "Python 3 is required to read Cargo's JSON artifact paths." >&2
    exit 1
fi

project_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
compiler="${1:-${KETOX_KOTLINC:-kotlinc}}"

# Python passes argument lists directly, preserving spaces in tool and artifact paths.
python3 - "$project_root" "$compiler" <<'PY'
import json
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(sys.argv[1])
compiler = shutil.which(sys.argv[2])
if compiler is None:
    bundled = root / ".tools/kotlin-2.2.0/kotlinc/bin/kotlinc"
    if bundled.exists():
        compiler = str(bundled)
if compiler is None:
    sys.exit("Kotlin compiler not found: " + sys.argv[2] + ". Install Kotlin 2.2.0 and put kotlinc on PATH, or pass its path as the first argument (or set KETOX_KOTLINC).")
compiler = str(Path(compiler).resolve())
cargo = shutil.which("cargo")
java = shutil.which("java")
if cargo is None:
    sys.exit("Cargo not found. Install Rust and put cargo on PATH.")
if java is None:
    sys.exit("Java not found. Install JDK 21 and put java on PATH.")

metadata_result = subprocess.run([cargo, "metadata", "--locked", "--format-version=1", "--no-deps"], cwd=root, stdout=subprocess.PIPE, text=True)
if metadata_result.returncode:
    sys.exit("cargo metadata failed.")
metadata = json.loads(metadata_result.stdout)
packages = [p for p in metadata["packages"] if p["name"] == "ketox-hello"]
if len(packages) != 1:
    sys.exit("Expected exactly one ketox-hello workspace package.")
package_id = packages[0]["id"]

print("Building the Ketox native example...", flush=True)
build = subprocess.run([cargo, "build", "--locked", "-p", "ketox-hello", "--message-format=json"], cwd=root, stdout=subprocess.PIPE, text=True)
messages = [json.loads(line) for line in build.stdout.splitlines() if line.strip()]
for message in messages:
    if message.get("reason") == "compiler-message":
        rendered = message["message"].get("rendered")
        if rendered:
            print(rendered, file=sys.stderr, end="")
if build.returncode:
    sys.exit("cargo build failed (exit %s)." % build.returncode)

output_dirs = {m["out_dir"] for m in messages if m.get("reason") == "build-script-executed" and m["package_id"] == package_id}
native_libraries = {filename for m in messages if m.get("reason") == "compiler-artifact" and m["package_id"] == package_id and "cdylib" in m["target"]["kind"] for filename in m["filenames"] if Path(filename).suffix in (".so", ".dylib", ".dll")}
if len(output_dirs) != 1:
    sys.exit("Cargo did not report exactly one generated output directory for ketox-hello.")
if len(native_libraries) != 1:
    sys.exit("Cargo did not report exactly one native library for ketox-hello.")

generated_kotlin = Path(output_dirs.pop()) / "RustApi.kt"
smoke_source = root / "integration-tests" / "jvm" / "Smoke.kt"
native_library = Path(native_libraries.pop())
for artifact in (generated_kotlin, smoke_source, native_library):
    if not artifact.is_file():
        sys.exit("Required artifact missing: " + str(artifact))
test_output = Path(metadata["target_directory"]) / "ketox-jvm"
test_output.mkdir(parents=True, exist_ok=True)
jar = test_output / "smoke.jar"

print("Compiling generated Kotlin and JVM integration checks...", flush=True)
compile_result = subprocess.run([compiler, str(generated_kotlin), str(smoke_source), "-jvm-target", "21", "-include-runtime", "-d", str(jar)], cwd=root)
if compile_result.returncode:
    sys.exit("Kotlin compilation failed (exit %s)." % compile_result.returncode)
print("Running JVM integration checks with -Xcheck:jni...", flush=True)
test_result = subprocess.run([java, "-Xcheck:jni", "-Djava.library.path=" + str(native_library.parent), "-cp", str(jar), "dev.ketox.example.SmokeKt"], cwd=root)
if test_result.returncode:
    sys.exit("JVM integration checks failed (exit %s)." % test_result.returncode)
print("Ketox JVM integration checks passed. Artifact: " + str(jar))
PY

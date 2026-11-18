#!/usr/bin/env bash
# Create the Python environment for a JConverter bundle.
#
# Usage: bootstrap_env.sh [--root DIR]
#
# Installs from $root/wheelhouse without network access when it is present,
# otherwise from engine/requirements.lock.txt. The venv is always created at
# $root/.venv because venvs are not relocatable.
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: bootstrap_env.sh [--root DIR] [--help]

Creates DIR/.venv for the JConverter bundle in DIR (default: the directory that
contains this script).
EOF
}

root=""
while [ $# -gt 0 ]; do
    case "$1" in
        --root)
            if [ $# -lt 2 ]; then
                printf 'error: --root needs a directory\n' >&2
                exit 2
            fi
            root="$2"
            shift 2
            ;;
        --root=*)
            root="${1#--root=}"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'error: unknown argument: %s\n' "$1" >&2
            exit 2
            ;;
    esac
done

if [ -z "$root" ]; then
    root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
root="$(cd "$root" && pwd)"

engine_dir="$root/engine"
venv_dir="$root/.venv"
venv_python="$venv_dir/bin/python"

fail() {
    printf '\nerror: %s\n' "$1" >&2
    exit 1
}

[ -f "$engine_dir/tt_engine/__main__.py" ] ||
    fail "the conversion engine is missing from $engine_dir. Reinstall JConverter."

# 1. Choose the interpreter: bundled runtime first, then the system Python.
python=""
for candidate in \
    "$root/runtime/python/bin/python3" \
    "$root/runtime/python/bin/python"
do
    if [ -x "$candidate" ]; then
        python="$candidate"
        break
    fi
done

if [ -z "$python" ]; then
    for candidate in python3 python; do
        if command -v "$candidate" >/dev/null 2>&1; then
            python="$(command -v "$candidate")"
            break
        fi
    done
fi

[ -n "$python" ] ||
    fail "Python could not be found. Reinstall JConverter."

version_ok="$("$python" -c 'import sys; print(int(sys.version_info >= (3, 11)))' 2>/dev/null || echo 0)"
[ "$version_ok" = "1" ] ||
    fail "the bundled Python runtime could not start. Reinstall JConverter."

printf 'Using Python: %s (%s)\n' "$python" "$("$python" --version 2>&1)"

# 2. Create the venv.
if [ ! -x "$venv_python" ]; then
    printf 'Creating %s\n' "$venv_dir"
    if ! "$python" -m venv "$venv_dir" 2>/dev/null; then
        rm -rf "$venv_dir"
        fail "could not create the Python environment (is python3-venv installed?)."
    fi
fi

# 3. Install the dependencies.
"$venv_python" -m pip install --upgrade --quiet pip >/dev/null 2>&1 || true

wheelhouse="$root/wheelhouse"
lockfile="$engine_dir/requirements.lock.txt"

if [ -d "$wheelhouse" ] && [ -n "$(ls -A "$wheelhouse" 2>/dev/null)" ]; then
    printf 'Installing dependencies from %s (offline)\n' "$wheelhouse"
    "$venv_python" -m pip install --quiet --no-index \
        --find-links "$wheelhouse" \
        -r "$lockfile"
elif [ -f "$lockfile" ]; then
    printf 'Installing dependencies from %s\n' "$lockfile"
    "$venv_python" -m pip install --quiet -r "$lockfile" ||
        fail "could not install the conversion dependencies."
else
    fail "the dependency list or wheelhouse is missing. Reinstall JConverter."
fi

# 4. Verify the engine imports and runs.
if ! PYTHONPATH="$engine_dir" "$venv_python" -c 'import tt_engine' >/dev/null 2>&1; then
    fail "the conversion engine could not be loaded. Reinstall JConverter."
fi

printf 'JConverter environment is ready: %s\n' "$venv_python"

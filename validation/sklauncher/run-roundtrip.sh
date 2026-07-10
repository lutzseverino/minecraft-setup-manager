#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$script_dir/versions.env"

state_dir=${SKLAUNCHER_VALIDATION_STATE_DIR:-/opt/sklauncher-validation}
probe_bin=${SKLAUNCHER_CONTRACT_PROBE:-}

if [ -z "$probe_bin" ] || [ ! -x "$probe_bin" ]; then
  echo "Set SKLAUNCHER_CONTRACT_PROBE to the validation probe executable." >&2
  exit 2
fi

case "$state_dir" in
  /|/opt|/home|/workspace) echo "Refusing unsafe validation state directory: $state_dir" >&2; exit 2 ;;
esac

artifacts_dir="$state_dir/artifacts"
build_dir="$state_dir/build"
workspace_dir="$state_dir/workspace"
results_dir="$state_dir/results"
jar_path="$artifacts_dir/SKlauncher-${SKLAUNCHER_VERSION}.jar"
image_name="minecraft-setup-manager/sklauncher-validation:${SKLAUNCHER_VERSION}"

mkdir -p "$artifacts_dir" "$build_dir" "$results_dir"
rm -rf "$workspace_dir" "$results_dir"/* "$build_dir"/*
mkdir -p "$workspace_dir/home" "$results_dir"
touch "$workspace_dir/home/.minecraft-setup-manager-sklauncher-sandbox"

if [ ! -f "$jar_path" ]; then
  curl --fail --location --proto '=https' --tlsv1.2 \
    --output "$jar_path.part" "$SKLAUNCHER_URL"
  mv "$jar_path.part" "$jar_path"
fi
echo "$SKLAUNCHER_SHA256  $jar_path" | sha256sum --check --strict

cp "$script_dir/Containerfile" "$build_dir/Containerfile"
cp "$script_dir/run-launcher.sh" "$build_dir/run-launcher.sh"
cp "$jar_path" "$build_dir/SKlauncher.jar"

docker build \
  --build-arg "TEMURIN_IMAGE=$TEMURIN_IMAGE" \
  --build-arg "SKLAUNCHER_SHA256=$SKLAUNCHER_SHA256" \
  --tag "$image_name" \
  --file "$build_dir/Containerfile" \
  "$build_dir"

run_launcher() {
  phase=$1
  docker run --rm \
    --name "msm-sklauncher-$phase" \
    --cap-drop ALL \
    --security-opt no-new-privileges:true \
    --pids-limit 512 \
    --memory 4g \
    --cpus 2 \
    --read-only \
    --tmpfs /tmp:rw,exec,nosuid,nodev,size=1g \
    --tmpfs /run:rw,nosuid,nodev,size=64m \
    --mount "type=bind,src=$workspace_dir,dst=/workspace" \
    --mount "type=bind,src=$results_dir,dst=/results" \
    "$image_name" "$phase"
}

run_launcher bootstrap
test -f "$workspace_dir/home/.minecraft/launcher_profiles.json"

HOME="$workspace_dir/home" \
MSM_SKLAUNCHER_VALIDATION=1 \
  "$probe_bin" write >"$results_dir/probe-write.json"

run_launcher reload

HOME="$workspace_dir/home" \
MSM_SKLAUNCHER_VALIDATION=1 \
  "$probe_bin" verify >"$results_dir/probe-verify.json"

{
  printf '{\n'
  printf '  "completedAt": "%s",\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '  "sklauncherVersion": "%s",\n' "$SKLAUNCHER_VERSION"
  printf '  "sklauncherSha256": "%s",\n' "$SKLAUNCHER_SHA256"
  printf '  "containerImage": "%s",\n' "$image_name"
  printf '  "containerBase": "%s"\n' "$TEMURIN_IMAGE"
  printf '}\n'
} >"$results_dir/run.json"

echo "SKlauncher round-trip validation passed. Evidence: $results_dir"

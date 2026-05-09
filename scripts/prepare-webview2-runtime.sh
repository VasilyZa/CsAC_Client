#!/usr/bin/env bash
set -euo pipefail

version="148.0.3967.54"
arch="x64"
runtime_name="Microsoft.WebView2.FixedVersionRuntime.${version}.${arch}"
download_url="https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/e2d62d9f-14bb-49fa-ac41-d3f96ddbd899/${runtime_name}.cab"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "${script_dir}/.." && pwd)"
tauri_dir="${repo_dir}/src-tauri"
runtime_dir="${tauri_dir}/webview2-fixed-runtime/${arch}"
cache_dir="${repo_dir}/target/webview2-cache"
cab_path="${cache_dir}/${runtime_name}.cab"
extract_dir="${cache_dir}/extract"

if [[ -f "${runtime_dir}/msedgewebview2.exe" ]]; then
  echo "WebView2 fixed runtime already exists: ${runtime_dir}"
  exit 0
fi

if ! command -v 7z >/dev/null 2>&1; then
  echo "7z is required to extract the WebView2 fixed runtime CAB." >&2
  exit 1
fi

mkdir -p "${cache_dir}"
if [[ ! -f "${cab_path}" ]]; then
  echo "Downloading WebView2 fixed runtime ${version} (${arch})..."
  curl -L --fail --retry 3 --retry-delay 2 -o "${cab_path}" "${download_url}"
fi

rm -rf "${extract_dir}" "${runtime_dir}"
mkdir -p "${extract_dir}" "${runtime_dir}"
7z x -y -o"${extract_dir}" "${cab_path}"

source_dir="${extract_dir}/${runtime_name}"
if [[ ! -f "${source_dir}/msedgewebview2.exe" ]]; then
  echo "Extracted WebView2 runtime is missing msedgewebview2.exe." >&2
  exit 1
fi

mv "${source_dir}/"* "${runtime_dir}/"
printf '%s\n' "${version}" > "${runtime_dir}/VERSION"
echo "Prepared WebView2 fixed runtime: ${runtime_dir}"

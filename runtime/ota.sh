#!/bin/bash
echo "Over the air updating..."

# Base URL for artifacts
base_url="https://gitlab.kscale.ai/zeroth-robotics/OpenLCH/-/jobs"

# List of artifact jobs with their job IDs
declare -A artifact_jobs=(
    ["build-runtime"]="119/artifacts/download"
    ["build-servo"]="119/artifacts/download"
    ["build-cviwrapper"]="119/artifacts/download"
)

# Directory to save the downloaded artifacts
download_dir="artifacts"
mkdir -p "$download_dir"

# Loop through the jobs and download each artifact
for job in "${!artifact_jobs[@]}"; do
    echo "Downloading $job..."
    curl -L -o "$download_dir/$job.zip" "$base_url/${artifact_jobs[$job]}"
    if [ $? -eq 0 ]; then
        echo "$job downloaded successfully."
    else
        echo "Failed to download $job. Please check the URL or your network connection."
    fi
done

echo "All artifacts downloaded."
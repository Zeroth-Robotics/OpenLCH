#!/bin/bash

PROJECT_ID=6
GITLAB_URL="https://gitlab.kscale.ai"
REF="main"
OUTPUT_DIR="./artifacts"
TOKEN=""

# Ensure unzip and jq are installed
for cmd in unzip jq; do
    if ! command -v $cmd &> /dev/null; then
        echo "$cmd could not be found. Please install it and retry."
        exit 1
    fi
done

gitlab_api_request() {
    local endpoint=$1
    if [ -n "$TOKEN" ]; then
        curl --silent --header "PRIVATE-TOKEN: $TOKEN" "$GITLAB_URL/api/v4/$endpoint"
    else
        curl --silent "$GITLAB_URL/api/v4/$endpoint"
    fi
}

echo "Fetching the latest pipeline for branch: $REF..."
pipeline_json=$(gitlab_api_request "projects/$PROJECT_ID/pipelines?ref=$REF&per_page=1")

pipeline_id=$(echo "$pipeline_json" | jq '.[0].id')
if [ -z "$pipeline_id" ]; then
    echo "Failed to fetch pipeline ID. Exiting."
    exit 1
fi
echo "Latest pipeline ID: $pipeline_id"

echo "Fetching jobs for pipeline ID: $pipeline_id..."
jobs_json=$(gitlab_api_request "projects/$PROJECT_ID/pipelines/$pipeline_id/jobs")

mkdir -p "$OUTPUT_DIR"

for job_name in "build-runtime" "build-servo" "build-cviwrapper"; do
    echo "Looking for job: $job_name..."
    job_id=$(echo "$jobs_json" | jq -r --arg NAME "$job_name" '.[] | select(.name == $NAME) | .id')

    if [ -n "$job_id" ]; then
        echo "Found $job_name with Job ID: $job_id. Downloading artifacts..."
        
        # Derive the target directory name by removing the 'build-' prefix
        target_dir=${job_name#build-}
        
        artifact_url="$GITLAB_URL/api/v4/projects/$PROJECT_ID/jobs/$job_id/artifacts"
        curl --silent --output "$OUTPUT_DIR/$target_dir.zip" "$artifact_url"

        if [ $? -ne 0 ] || [ ! -s "$OUTPUT_DIR/$target_dir.zip" ]; then
            echo "Failed to download artifacts for $job_name. Skipping extraction."
            continue
        fi

        echo "$job_name artifacts downloaded to $OUTPUT_DIR/$target_dir.zip"

        echo "Extracting $target_dir.zip..."
        unzip -o "$OUTPUT_DIR/$target_dir.zip" -d "$OUTPUT_DIR/$target_dir"
        if [ $? -eq 0 ]; then
            echo "$target_dir.zip extracted to $OUTPUT_DIR/$target_dir/"
            # Optional: Remove the ZIP file after extraction
            # rm "$OUTPUT_DIR/$target_dir.zip"
        else
            echo "Failed to extract $target_dir.zip. It may be corrupted."
        fi
    else
        echo "Job $job_name not found in this pipeline."
    fi
done

echo "Artifacts download and extraction completed."

# Upload to Milk-V Duo
echo "Uploading to Milk-V Duo..."
scp -v -r -O artifacts/runtime/runtime/target/riscv64gc-unknown-linux-musl/release/* root@192.168.42.1:/usr/local/bin/

echo "Upload completed."
#!/bin/bash

# Function to print usage
print_usage() {
    echo "Usage: MODEL_PATH=/path/to/your/model.pt [MODEL_SHAPE='1,615'] ./convert_model.sh [model_shape]"
    echo "  MODEL_PATH: Path to your .pt model file (required)"
    echo "  MODEL_SHAPE: Shape of the input tensor (optional, can be set as env var or command-line arg)"
    echo "               Default is '1,615' if not specified"
}

# Check if MODEL_PATH is set
if [ -z "$MODEL_PATH" ]; then
    echo "Error: MODEL_PATH environment variable is not set."
    print_usage
    exit 1
fi

# Set default model shape
MODEL_SHAPE=${MODEL_SHAPE:-"1,615"}

# Override MODEL_SHAPE if provided as command-line argument
if [ $# -eq 1 ]; then
    MODEL_SHAPE=$1
fi

# Extract directory, filename, and name without extension
MODEL_DIR=$(dirname "$MODEL_PATH")
MODEL_FILENAME=$(basename "$MODEL_PATH")
MODEL_NAME="${MODEL_FILENAME%.*}"
ARTIFACTS_DIR="${MODEL_DIR}/${MODEL_NAME}_artifacts"

# Create artifacts directory and copy the original .pt file
mkdir -p "$ARTIFACTS_DIR"
cp "$MODEL_PATH" "$ARTIFACTS_DIR/"

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Run the Docker container
docker run --rm --privileged --name DuoTPU \
    -v "$ARTIFACTS_DIR":/workspace \
    -v "$SCRIPT_DIR/tpu-mlir":/tpu-mlir \
    -it sophgo/tpuc_dev:v3.1 /bin/bash -c "
    # Source the environment setup script
    source /tpu-mlir/envsetup.sh

    # Change to the workspace directory
    cd /workspace

    # Step 1: Convert .pt to .mlir
    model_transform.py --model_name $MODEL_NAME --model_def $MODEL_FILENAME --input_shapes [[$MODEL_SHAPE]] --mlir $MODEL_NAME.mlir

    # Step 2: Convert .mlir to .cvimodel
    model_deploy.py --mlir $MODEL_NAME.mlir --quantize BF16 --chip cv181x --model $MODEL_NAME.cvimodel

    # Set permissions so that the host user can access the generated files
    chmod 666 $MODEL_NAME.mlir $MODEL_NAME.cvimodel
"

echo "Conversion complete. Output files are in $ARTIFACTS_DIR"

#!/bin/bash

# Download whisper.cpp models
MODEL_DIR="$HOME/.local/share/y2md/models"

# Models to download
MODELS=(
    "ggml-base.en.bin"
    "ggml-base.bin"
)

mkdir -p "$MODEL_DIR"

for MODEL in "${MODELS[@]}"; do
    MODEL_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/$MODEL"
    MODEL_PATH="$MODEL_DIR/$MODEL"
    
    if [ ! -f "$MODEL_PATH" ]; then
        echo "Downloading Whisper model: $MODEL"
        wget -O "$MODEL_PATH" "$MODEL_URL"
        echo "Model downloaded to: $MODEL_PATH"
    else
        echo "Model already exists: $MODEL_PATH"
    fi
done

echo "All models downloaded successfully"
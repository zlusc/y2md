#!/bin/bash

set -e

# Download whisper.cpp models and setup Ollama
MODEL_DIR="$HOME/.local/share/y2md/models"

# Whisper models to download
MODELS=(
    "ggml-base.en.bin"
    "ggml-base.bin"
)

# LLM model for enhanced formatting
LLM_MODEL="mistral-nemo:12b-instruct-2407-q5_0"

echo "Setting up YouTube to Markdown Transcriber..."

# Check for yt-dlp
if ! command -v yt-dlp &> /dev/null; then
    echo "ERROR: yt-dlp is not installed!"
    echo ""
    echo "Please install yt-dlp using one of these methods:"
    echo "  - pip: python3 -m pip install yt-dlp"
    echo "  - Arch Linux: sudo pacman -S yt-dlp"
    echo "  - Ubuntu/Debian: sudo apt install yt-dlp"
    echo "  - macOS: brew install yt-dlp"
    echo "  - Or download standalone: sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp"
    echo ""
    exit 1
else
    echo "✓ yt-dlp is installed"
fi

# Check for FFmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo "WARNING: FFmpeg is not installed. Audio conversion may fail."
    echo "  Install FFmpeg: sudo apt install ffmpeg (Ubuntu/Debian) or brew install ffmpeg (macOS)"
else
    echo "✓ FFmpeg is installed"
fi

# Create model directory
mkdir -p "$MODEL_DIR"

# Download Whisper models
echo "Downloading Whisper speech-to-text models..."
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

echo "Whisper models downloaded successfully"

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo "Installing Ollama for local LLM support..."
    curl -fsSL https://ollama.ai/install.sh | sh
    echo "Ollama installed successfully"
else
    echo "Ollama is already installed"
fi

# Start Ollama service if not running
if ! systemctl --user is-active --quiet ollama 2>/dev/null; then
    echo "Starting Ollama service..."
    systemctl --user start ollama
    # Wait for service to be ready
    sleep 5
fi

# Check if Ollama is responding
if ollama list &> /dev/null; then
    echo "Ollama service is running"
else
    echo "Starting Ollama service..."
    ollama serve &
    sleep 10
fi

# Download LLM model if not present
echo "Checking for LLM model: $LLM_MODEL"
if ollama list | grep -q "$LLM_MODEL"; then
    echo "LLM model already exists: $LLM_MODEL"
else
    echo "Downloading LLM model: $LLM_MODEL"
    ollama pull "$LLM_MODEL"
    echo "LLM model downloaded successfully"
fi

echo ""
echo "Setup completed successfully!"
echo ""
echo "To use LLM-enhanced formatting, run:"
echo "  cargo run -- <YOUTUBE_URL> --use-llm"
echo ""
echo "To start Ollama service manually:"
echo "  systemctl --user start ollama"
echo ""
echo "To check Ollama status:"
echo "  systemctl --user status ollama"
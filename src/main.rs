use clap::Parser;
use std::fs;
use y2md::{fetch_video_metadata, format_markdown, transcribe_video, validate_youtube_url};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// YouTube URL to transcribe
    url: String,

    /// Output directory for transcript
    #[arg(short, long, default_value = ".")]
    out_dir: String,

    /// Prefer captions over STT
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    prefer_captions: bool,

    /// Language code override
    #[arg(long)]
    lang: Option<String>,

    /// Include timestamps in transcript
    #[arg(long, default_value_t = false)]
    timestamps: bool,

    /// Compact output format
    #[arg(long, default_value_t = false)]
    compact: bool,

    /// Paragraph length for enhanced formatting (sentences per paragraph)
    #[arg(long, default_value_t = 4)]
    paragraph_length: usize,

    /// Force enhanced formatting even for music content
    #[arg(long, default_value_t = false)]
    force_formatting: bool,

    /// Cookies file for restricted content
    #[arg(long)]
    cookies: Option<String>,

    /// Whisper model size
    #[arg(long, default_value = "small")]
    model: String,

    /// Number of threads for STT
    #[arg(long, default_value_t = 4)]
    threads: usize,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Use local LLM (Ollama) for enhanced transcript formatting
    #[arg(long, default_value_t = false)]
    use_llm: bool,

    /// Dry run - don't write files
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        tracing_subscriber::fmt::init();
    }

    // Validate URL and extract video ID
    let video_id = validate_youtube_url(&args.url)?;

    // Fetch video metadata
    let metadata = fetch_video_metadata(&video_id).await?;

    println!("Transcribing: {}", metadata.title);
    println!(
        "Channel: {}",
        metadata.channel.as_deref().unwrap_or("Unknown")
    );
    println!("Video ID: {}", video_id);
    println!("Output directory: {}", args.out_dir);

    // Perform transcription
    let (transcript, source) = transcribe_video(
        &video_id,
        args.prefer_captions,
        args.lang.as_deref(),
        &args.out_dir,
        args.paragraph_length,
        args.force_formatting,
    )
    .await?;

    // Format as Markdown
    let markdown = format_markdown(
        &metadata,
        &transcript,
        &source,
        args.timestamps,
        args.compact,
        args.paragraph_length,
        args.use_llm,
    )
    .await;

    // Generate filename
    let sanitized_title = metadata
        .title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let filename = format!(
        "{}_{}_{}.md",
        chrono::Utc::now().format("%Y-%m-%d"),
        video_id,
        sanitized_title
    );
    let output_path = std::path::Path::new(&args.out_dir).join(&filename);

    if args.dry_run {
        println!("Dry run - would save to: {}", output_path.display());
        println!(
            "Markdown preview (first 500 chars):\n{}",
            &markdown[..markdown.len().min(500)]
        );
    } else {
        // Save to file
        fs::write(&output_path, &markdown)?;
        println!("Transcription saved to: {}", output_path.display());
    }

    // Calculate formatting statistics
    let word_count = transcript.split_whitespace().count();
    let char_count = transcript.chars().count();
    let paragraph_count = markdown.matches("\n\n").count() + 1;
    let formatting_type = if args.compact { "compact" } else { "enhanced" };

    println!("Transcription completed using: {}", source);
    println!("Formatting statistics:");
    println!("  - Formatting type: {}", formatting_type);
    println!("  - Paragraph length: {} sentences", args.paragraph_length);
    println!("  - Word count: {}", word_count);
    println!("  - Character count: {}", char_count);
    println!("  - Paragraph count: {}", paragraph_count);

    Ok(())
}

use y2md::{format_transcript, clean_transcript, format_paragraphs};

fn main() {
    // Test with sample transcript text
    let sample_transcript = "hello world this is a test sentence how are you doing today i hope you are doing well this is another test sentence to demonstrate the formatting capabilities of our system";
    
    println!("Original transcript:");
    println!("{}", sample_transcript);
    println!();
    
    println!("After clean_transcript:");
    let cleaned = clean_transcript(sample_transcript);
    println!("{}", cleaned);
    println!();
    
    println!("After format_paragraphs (enhanced mode):");
    let formatted_enhanced = format_paragraphs(&cleaned, 4);
    println!("{}", formatted_enhanced);
    println!();
    
    println!("After format_transcript (compact mode):");
    let formatted_compact = format_transcript(sample_transcript, true);
    println!("{}", formatted_compact);
    println!();
    
    println!("After format_transcript (enhanced mode):");
    let formatted_enhanced = format_transcript(sample_transcript, false);
    println!("{}", formatted_enhanced);
}
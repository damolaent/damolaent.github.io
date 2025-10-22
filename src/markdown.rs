use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use slug::slugify;
use std::fs;
use regex::Regex;
use image::ImageReader;
use reqwest;
use image::GenericImageView;
use std::time::Duration;

/// Front matter for a typical blog post (includes date).
#[derive(Clone, Debug, Deserialize)]
pub struct PostFrontMatter {
    pub title: String,
    pub date: String,
    pub author: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// Front matter for a generic page (like About).
#[derive(Debug, Deserialize)]
pub struct PageFrontMatter {
    pub title: String,
    pub author: String,
    pub description: Option<String>,
}

/// Represents a single blog post.
#[derive(Clone, Debug)]
pub struct Post {
    /// Parsed front matter (title, date, author).
    pub front_matter: PostFrontMatter,
    /// Final HTML content after Markdown conversion.
    pub content: String,
    /// Estimated reading time (in minutes).
    pub reading_time: usize,
    /// Destination file name (e.g. "docs/posts/my-title.html").
    pub file_name: String,
}

/// Represents a generic page (e.g., About page).
#[derive(Debug)]
pub struct Page {
    /// Parsed front matter (title, author).
    pub front_matter: PageFrontMatter,
    /// Final HTML content after Markdown conversion.
    pub content: String,
}

/// Parses a blog post Markdown file with front matter:
///
/// ```md
/// ---
/// title: "My Post"
/// date: "2025-01-30"
/// author: "John Doe"
/// ---
///
/// # My Post Content
/// ```

pub fn parse_post_markdown(file_path: &str) -> Result<Post, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;

    // 1. Split off the leading '---\n'
    let mut sections = content.splitn(2, "---\n");
    sections.next(); // skip the first empty part if any

    // 2. Extract front matter + remainder
    let front_matter_str = sections
        .next()
        .ok_or("Missing front matter section (--- line not found)")?;

    // 3. Split front matter from the actual Markdown body
    let mut body_sections = front_matter_str.splitn(2, "\n---\n");
    let front_matter_yaml = body_sections
        .next()
        .ok_or("Missing YAML front matter contents")?;
    let markdown_body = body_sections
        .next()
        .ok_or("Missing Markdown body after front matter")?;

    // 4. Parse front matter with Serde
    let front_matter: PostFrontMatter = serde_yaml::from_str(front_matter_yaml)?;

    // 5. Convert Markdown to HTML
    let mut html_output = String::new();
    let parser = Parser::new_ext(markdown_body, Options::all());
    html::push_html(&mut html_output, parser);

    let html_output = html_output.replace(
        "<pre><code class=\"language-",
        "<pre class=\"line-numbers\"><code class=\"language-",
    );

    // 6. Calculate estimated reading time (assume ~200 words/min)
    let word_count = markdown_body.split_whitespace().count();
    let reading_time = (word_count as f64 / 200.0).ceil() as usize;

    // 7. Generate a default file name in `docs/posts`
    let slug = slugify(&front_matter.title);
    let file_name = format!("docs/posts/{}.html", slug);

    // after you generate html_output
    let img_tag_re = Regex::new(r#"<img\s+[^>]*src="([^"]+)"\s+alt="([^"]*)".*?/?>"#)?;

    let rewritten_html = img_tag_re.replace_all(&html_output, |caps: &regex::Captures| {
        let src = &caps[1];
        let alt = &caps[2];

        if src.starts_with("http") {
            let client = reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap();

            match client.get(src).send() {
                Ok(response) if response.status().is_success() => {
                    if let Ok(bytes) = response.bytes() {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let dims = img.dimensions();
                            return format!(
                                r#"<div class='shimmer aspect-ratio' style='--aspect-ratio:{} / {}'><img src="{}" alt="{}"/></div>"#,
                                dims.0, dims.1, src, alt
                            );
                        }
                    }
                }
                _ => {}
            }

            println!("Could not retrieve or decode remote image '{}', continuing without image", src);
            return "".to_string();
        }

        // local
        let cleaned_src = src.trim_start_matches("../");
        let src_path = std::path::Path::new("docs").join(cleaned_src);

        if let Ok(img) = ImageReader::open(&src_path) {
            let dims = img.into_dimensions().unwrap_or((0, 0));
            return format!(
                r#"<div class='shimmer aspect-ratio' style='--aspect-ratio:{} / {}'><img src="{}" alt="{}"/></div>"#,
                dims.0, dims.1, src, alt
            );
        }

        println!("Could not open local image '{}', continuing without image", src);
        "".to_string()
    }).to_string();

    Ok(Post {
        front_matter,
        content: rewritten_html,
        reading_time,
        file_name,
    })
}

/// Parses a generic Markdown page with simpler front matter:
///
/// ```md
/// ---
/// title: "About Me"
/// author: "John Doe"
/// ---
///
/// # About Content Here
/// ```
pub fn parse_page_markdown(file_path: &str) -> Result<Page, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;

    // 1. Split off the leading '---\n'
    let mut sections = content.splitn(2, "---\n");
    sections.next(); // skip the first empty part if any

    // 2. Extract front matter + remainder
    let front_matter_str = sections
        .next()
        .ok_or("Missing front matter section (--- line not found)")?;

    // 3. Split front matter from the actual Markdown body
    let mut body_sections = front_matter_str.splitn(2, "\n---\n");
    let front_matter_yaml = body_sections
        .next()
        .ok_or("Missing YAML front matter contents")?;
    let markdown_body = body_sections
        .next()
        .ok_or("Missing Markdown body after front matter")?;

    // 4. Parse front matter with Serde
    let front_matter: PageFrontMatter = serde_yaml::from_str(front_matter_yaml)?;

    // 5. Convert Markdown to HTML
    let mut html_output = String::new();
    let parser = Parser::new_ext(markdown_body, Options::all());
    html::push_html(&mut html_output, parser);

    let html_output = html_output.replace(
        "<pre><code class=\"language-",
        "<pre class=\"line-numbers\"><code class=\"language-",
    );

    // after you generate html_output
    let img_tag_re = Regex::new(r#"<img\s+[^>]*src="([^"]+)"\s+alt="([^"]*)".*?/?>"#)?;

    let rewritten_html = img_tag_re.replace_all(&html_output, |caps: &regex::Captures| {
        let src = &caps[1];
        let alt = &caps[2];

        if src.starts_with("http") {
            let client = reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap();

            match client.get(src).send() {
                Ok(response) if response.status().is_success() => {
                    if let Ok(bytes) = response.bytes() {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let dims = img.dimensions();
                            return format!(
                                r#"<div class='shimmer aspect-ratio' style='--aspect-ratio:{} / {}'><img src="{}" alt="{}"/></div>"#,
                                dims.0, dims.1, src, alt
                            );
                        }
                    }
                }
                _ => {}
            }

            println!("Could not retrieve or decode remote image '{}', continuing without image", src);
            return "".to_string();
        }

        // local
        let cleaned_src = src.trim_start_matches("../");
        let src_path = std::path::Path::new("docs").join(cleaned_src);

        if let Ok(img) = ImageReader::open(&src_path) {
            let dims = img.into_dimensions().unwrap_or((0, 0));
            return format!(
                r#"<div class='shimmer aspect-ratio' style='--aspect-ratio:{} / {}'><img src="{}" alt="{}"/></div>"#,
                dims.0, dims.1, src, alt
            );
        }

        println!("Could not open local image '{}', continuing without image", src);
        "".to_string()
    }).to_string();

    Ok(Page {
        front_matter,
        content: rewritten_html,
    })
}

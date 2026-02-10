use super::events::EmbedInfo;

/// Extract all URLs (http/https) from message content (max 5).
pub fn extract_urls(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for word in content.split_whitespace() {
        if word.starts_with("http://") || word.starts_with("https://") {
            // Trim trailing punctuation that's likely not part of the URL
            let url = word.trim_end_matches(['>', ')', ']', ',', '.', ';']);
            urls.push(url.to_string());
            if urls.len() >= 5 {
                break;
            }
        }
    }
    urls
}

/// Fetch Open Graph metadata for a URL.
/// Returns None if the fetch fails or no OG tags are found.
pub async fn unfurl_url(client: &reqwest::Client, url: &str) -> Option<EmbedInfo> {
    let resp = client
        .get(url)
        .header("User-Agent", "ConcordBot/1.0 (link preview)")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .ok()?;

    // Only parse HTML responses
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.contains("text/html") {
        return None;
    }

    // Limit body read to 256KB to avoid abuse
    let body = resp.text().await.ok()?;
    let body = if body.len() > 256 * 1024 {
        &body[..256 * 1024]
    } else {
        &body
    };

    let title = extract_meta(body, "og:title")
        .or_else(|| extract_html_title(body));
    let description = extract_meta(body, "og:description")
        .or_else(|| extract_meta(body, "description"));
    let image_url = extract_meta(body, "og:image");
    let site_name = extract_meta(body, "og:site_name");

    // Must have at least a title to be useful
    if title.is_none() && description.is_none() {
        return None;
    }

    Some(EmbedInfo {
        url: url.to_string(),
        title,
        description,
        image_url,
        site_name,
    })
}

/// Extract content from a <meta property="..." content="..."> or <meta name="..." content="..."> tag.
fn extract_meta(html: &str, name: &str) -> Option<String> {
    let patterns = [
        format!(r#"property="{name}""#),
        format!(r#"property='{name}'"#),
        format!(r#"name="{name}""#),
        format!(r#"name='{name}'"#),
    ];

    for pattern in &patterns {
        if let Some(pos) = html.find(pattern.as_str()) {
            let search_end = (pos + 500).min(html.len());
            let slice = &html[pos..search_end];

            if let Some(content) = extract_content_attr(slice) {
                let decoded = html_decode(&content);
                if !decoded.is_empty() {
                    return Some(decoded);
                }
            }
        }
    }

    None
}

/// Extract the value of a content="..." attribute from a tag fragment.
fn extract_content_attr(tag_fragment: &str) -> Option<String> {
    if let Some(start) = tag_fragment.find("content=\"") {
        let value_start = start + 9;
        if let Some(end) = tag_fragment[value_start..].find('"') {
            return Some(tag_fragment[value_start..value_start + end].to_string());
        }
    }
    if let Some(start) = tag_fragment.find("content='") {
        let value_start = start + 9;
        if let Some(end) = tag_fragment[value_start..].find('\'') {
            return Some(tag_fragment[value_start..value_start + end].to_string());
        }
    }
    None
}

/// Extract <title>...</title> as fallback.
fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title")?.checked_add(6)?;
    let after_tag = lower[start..].find('>')?;
    let content_start = start + after_tag + 1;
    let end = lower[content_start..].find("</title>")?;
    let title = html[content_start..content_start + end].trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some(html_decode(&title))
    }
}

/// Decode basic HTML entities.
fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

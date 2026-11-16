//! Translation providers.
//!
//! * `google`  – Google Translate's public web endpoint. Fast, free, no key.
//! * `llm`     – any OpenAI-compatible chat-completions endpoint
//!               (OpenAI, Anthropic-compatible proxies, Ollama, LM Studio,
//!               OpenRouter…). Supports style/context prompts for
//!               higher-quality, domain-aware translation.

use crate::config::Config;
use crate::http;
use crate::util::percent_encode;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    Natural,
    Academic,
    Tech,
    Legal,
    Short,
}

impl Style {
    pub fn from_str(s: &str) -> Style {
        match s {
            "academic" => Style::Academic,
            "tech" => Style::Tech,
            "legal" => Style::Legal,
            "short" => Style::Short,
            _ => Style::Natural,
        }
    }

    fn instruction(self) -> &'static str {
        match self {
            Style::Natural => "Translate into natural, idiomatic fluent text. Do not be literal.",
            Style::Academic => "Translate into academic register. Preserve technical terminology \
                              precisely, keep formal tone and complete sentences.",
            Style::Tech => "Translate as a senior software engineer would: keep identifiers, code \
                           snippets, API names and command lines untranslated, translate comments \
                           and prose accurately.",
            Style::Legal => "Translate as a legal professional. Preserve defined terms exactly, \
                             keep modal verbs and obligations precise, do not add or drop clauses.",
            Style::Short => "Translate concisely. Keep it short and to the point, preserving meaning.",
        }
    }
}

/// Translate `text` from `src` to `dst` using the configured provider.
pub fn translate(text: &str, cfg: &Config) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(String::new());
    }
    if cfg.provider == "llm" && !cfg.llm_key.is_empty() {
        match translate_llm(text, cfg) {
            Ok(t) => return Ok(t),
            Err(e) => {
                crate::util::log(&format!("LLM provider failed ({e}); falling back to Google"));
            }
        }
    }
    translate_google(text, &cfg.source_lang, &cfg.target_lang)
}

// ---------------------------------------------------------------- Google

fn translate_google(text: &str, src: &str, dst: &str) -> Result<String, String> {
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&tl={}&dt=t&q={}",
        percent_encode(src),
        percent_encode(dst),
        percent_encode(text)
    );
    let body = http::get(&url)?;
    parse_google_json(&body)
}

fn parse_google_json(body: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| format!("bad response from translator: {e}"))?;
    let arr = v
        .get(0)
        .and_then(|x| x.as_array())
        .ok_or("unexpected translator response")?;
    let mut out = String::new();
    for seg in arr {
        if let Some(s) = seg.get(0).and_then(|x| x.as_str()) {
            out.push_str(s);
        }
    }
    if out.is_empty() {
        return Err("translator returned an empty result".into());
    }
    Ok(out)
}

// ------------------------------------------------------------------- LLM

fn translate_llm(text: &str, cfg: &Config) -> Result<String, String> {
    let style = Style::from_str(&cfg.llm_style);
    let src = if cfg.source_lang == "auto" {
        "the source language".to_string()
    } else {
        cfg.source_lang.clone()
    };

    let system = format!(
        "You are a professional translator. Translate the user's text from {} to {}.\n\
         {}\n\
         Output ONLY the translation. No explanations, no notes, no original text, \
         no markdown fences.",
        src,
        cfg.target_lang,
        style.instruction()
    );

    let body = serde_json::json!({
        "model": cfg.llm_model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": text}
        ],
        "temperature": 0.2,
        "stream": false
    });

    let url = format!("{}/chat/completions", cfg.llm_endpoint.trim_end_matches('/'));
    let resp = http::post_json(&url, &body.to_string(), &[("Authorization".into(), format!("Bearer {}", cfg.llm_key))])?;

    let v: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|e| format!("bad LLM response: {e}"))?;
    if let Some(err) = v.get("error") {
        return Err(format!("LLM error: {}", err));
    }
    let content = v
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("no content in LLM response")?;
    Ok(clean_llm_output(content))
}

fn clean_llm_output(s: &str) -> String {
    let t = s.trim();
    let t = if t.starts_with("```") {
        // strip a leading code fence if the model ignored the instruction
        let body = t.trim_start_matches("```");
        match body.split_once('\n') {
            Some((_, rest)) => rest.trim_end_matches("```").trim(),
            None => body,
        }
    } else {
        t
    };
    t.to_string()
}

/// Detect a likely source language code from a short sample of text.
/// Returns an ISO code such as "en", "vi", "ja", or "auto".
pub fn detect_lang(sample: &str) -> String {
    let s = sample.trim();
    if s.is_empty() {
        return "auto".into();
    }
    // Vietnamese diacritics
    if s.chars().any(|c| matches!(c, 'ă'|'â'|'đ'|'ê'|'ô'|'ơ'|'ư'|'à'|'á'|'ả'|'ã'|'ạ')) {
        return "vi".into();
    }
    // CJK
    if s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)) {
        return "zh".into();
    }
    if s.chars().any(|c| ('\u{3040}'..='\u{30FF}').contains(&c)) {
        return "ja".into();
    }
    if s.chars().any(|c| ('\u{AC00}'..='\u{D7AF}').contains(&c)) {
        return "ko".into();
    }
    // Cyrillic
    if s.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)) {
        return "ru".into();
    }
    // Latin script, accented → likely a European language; default English
    let letters = s.chars().filter(|c| c.is_alphabetic()).count();
    if letters == 0 {
        return "auto".into();
    }
    "auto".into()
}

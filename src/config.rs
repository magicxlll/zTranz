use serde_json::Value;

#[derive(Clone)]
pub struct Config {
    pub source_lang: String,
    pub target_lang: String,
    pub provider: String,     // "google" | "llm"
    pub llm_endpoint: String, // OpenAI-compatible base, e.g. https://api.openai.com/v1
    pub llm_model: String,
    pub llm_key: String,
    pub llm_style: String, // "natural" | "academic" | "tech" | "legal" | "short"
    pub auto_lang: bool,
    pub region: Option<(i32, i32, i32, i32)>, // last capture region, screen coords
}

impl Config {
    pub fn load() -> Self {
        let path = crate::util::config_path();
        let mut c = Self::default();
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                c = c.merge(v);
            }
        }
        c
    }

    /// Persist to disk. Reserved for a future settings UI.
    #[allow(dead_code)]
    pub fn save(&self) {
        let path = crate::util::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(
            &path,
            serde_json::to_string_pretty(self).unwrap_or_default(),
        );
    }

    fn merge(mut self, v: Value) -> Self {
        let get = |k: &str| v.get(k).cloned();
        if let Some(s) = get("source_lang").and_then(|x| x.as_str().map(str::to_string)) {
            self.source_lang = s;
        }
        if let Some(s) = get("target_lang").and_then(|x| x.as_str().map(str::to_string)) {
            self.target_lang = s;
        }
        if let Some(s) = get("provider").and_then(|x| x.as_str().map(str::to_string)) {
            self.provider = s;
        }
        if let Some(s) = get("llm_endpoint").and_then(|x| x.as_str().map(str::to_string)) {
            self.llm_endpoint = s;
        }
        if let Some(s) = get("llm_model").and_then(|x| x.as_str().map(str::to_string)) {
            self.llm_model = s;
        }
        if let Some(s) = get("llm_key").and_then(|x| x.as_str().map(str::to_string)) {
            self.llm_key = s;
        }
        if let Some(s) = get("llm_style").and_then(|x| x.as_str().map(str::to_string)) {
            self.llm_style = s;
        }
        if let Some(b) = get("auto_lang").and_then(|x| x.as_bool()) {
            self.auto_lang = b;
        }
        if let Some(arr) = get("region") {
            if let Some(r) = arr.as_array() {
                if r.len() == 4 {
                    self.region = Some((
                        r[0].as_i64().unwrap_or(0) as i32,
                        r[1].as_i64().unwrap_or(0) as i32,
                        r[2].as_i64().unwrap_or(0) as i32,
                        r[3].as_i64().unwrap_or(0) as i32,
                    ));
                }
            }
        }
        self
    }

    fn default() -> Self {
        Self {
            source_lang: "auto".into(),
            target_lang: "vi".into(),
            provider: "google".into(),
            llm_endpoint: "https://api.openai.com/v1".into(),
            llm_model: "gpt-4o-mini".into(),
            llm_key: String::new(),
            llm_style: "natural".into(),
            auto_lang: true,
            region: None,
        }
    }
}

impl serde::Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut st = serializer.serialize_struct("Config", 9)?;
        st.serialize_field("source_lang", &self.source_lang)?;
        st.serialize_field("target_lang", &self.target_lang)?;
        st.serialize_field("provider", &self.provider)?;
        st.serialize_field("llm_endpoint", &self.llm_endpoint)?;
        st.serialize_field("llm_model", &self.llm_model)?;
        st.serialize_field("llm_key", &self.llm_key)?;
        st.serialize_field("llm_style", &self.llm_style)?;
        st.serialize_field("auto_lang", &self.auto_lang)?;
        match self.region {
            Some((x, y, w, h)) => st.serialize_field("region", &serde_json::json!([x, y, w, h]))?,
            None => st.serialize_field("region", &Value::Null)?,
        }
        st.end()
    }
}

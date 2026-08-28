use reqwest::blocking::{multipart, Client};
use serde::Deserialize;

use super::{ProviderError, TranscriptionProvider, TranscriptionRequest};

/// Implements TranscriptionProvider against any endpoint that speaks
/// OpenAI's /audio/transcriptions multipart schema - Groq, OpenAI itself,
/// and self-hosted/custom endpoints that mirror it.
pub struct OpenAiCompatibleProvider {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: Option<String>,
}

impl TranscriptionProvider for OpenAiCompatibleProvider {
    fn transcribe(&self, req: &TranscriptionRequest) -> Result<String, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::MissingApiKey);
        }

        let file_part = multipart::Part::file(req.audio_path).map_err(|e| {
            ProviderError::UnexpectedResponseShape(format!("failed to read audio file: {e}"))
        })?;

        let mut form = multipart::Form::new()
            .part("file", file_part)
            .text("model", req.model.to_string())
            .text("prompt", req.prompt.to_string());

        if let Some(lang) = req.language {
            form = form.text("language", lang.to_string());
        }

        let url = format!(
            "{}/audio/transcriptions",
            self.base_url.trim_end_matches('/')
        );

        let response = Client::new()
            .post(&url)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .map_err(ProviderError::Network)?;

        let status = response.status();
        let body = response.text().map_err(ProviderError::Network)?;

        if !status.is_success() {
            return Err(ProviderError::Http {
                status: status.as_u16(),
                body,
            });
        }

        let parsed: TranscriptionResponse = serde_json::from_str(&body)
            .map_err(|_| ProviderError::UnexpectedResponseShape(body.clone()))?;

        // The API always returns a leading space; trim it (was previously
        // done with `sed 's/^ //'`).
        parsed
            .text
            .map(|t| t.trim_start().to_string())
            .ok_or(ProviderError::UnexpectedResponseShape(body))
    }
}

use base64::engine::general_purpose;

use adk_gemini::Model::Gemini25Flash;
use adk_gemini::{Gemini, GeminiBuilder};
use base64::Engine;
use bytes::Bytes;
use serde::Serialize;
use std::error::Error;

pub struct AiUtils {
    gemini: Gemini,
}

#[derive(Debug, Serialize)]
pub struct AiResponse {
    content: String,
    thought: String,
}

impl AiUtils {
    pub fn new(gemini_api_key: &str) -> Self {
        Self {
            gemini: GeminiBuilder::new(gemini_api_key)
                .with_model(Gemini25Flash)
                .build()
                .expect("Gemini should be build"),
        }
    }

    pub async fn prompt(&self, prompt: &str) -> Result<AiResponse, Box<dyn Error>> {
        match self
            .gemini
            .generate_content()
            .with_user_message(prompt)
            .with_dynamic_thinking()
            .with_thoughts_included(true)
            .execute()
            .await
        {
            Ok(s) => {
                let thought: String = s.thoughts().join("\n").to_string();
                let content: String = s.text();

                Ok(AiResponse { thought, content })
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn prompt_with_image(
        &self,
        prompt: &str,
        data: Bytes,
    ) -> Result<AiResponse, Box<dyn Error>> {
        let content_type: &str = file_type::FileType::from_bytes(&data).media_types()[0];

        let encoded: String = general_purpose::STANDARD.encode(&data);

        match self
            .gemini
            .generate_content()
            .with_user_message(prompt)
            .with_dynamic_thinking()
            .with_thoughts_included(true)
            .with_inline_data(encoded, content_type)
            .execute()
            .await
        {
            Ok(s) => {
                let thought: String = s.thoughts().join("\n").to_string();
                let content: String = s.text();

                Ok(AiResponse { thought, content })
            }
            Err(e) => Err(e.into()),
        }
    }
}

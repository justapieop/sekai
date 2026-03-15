use adk_gemini::Gemini;
use std::error::Error;

pub struct AiUtils {
    gemini: Gemini,
}

impl AiUtils {
    pub fn new(gemini_api_key: &str) -> Self {
        Self {
            gemini: Gemini::pro(gemini_api_key).expect("GEMINI_API_KEY should be valid"),
        }
    }

    pub async fn prompt(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        match self
            .gemini
            .generate_content()
            .with_user_message(prompt)
            .with_dynamic_thinking()
            .execute()
            .await
        {
            Ok(s) => Ok(s.text()),
            Err(e) => return Err(e.into()),
        }
    }
}

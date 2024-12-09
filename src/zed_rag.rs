use zed::{
    http_client::{HttpMethod, HttpRequest},
    serde_json::{self, json},
};
use zed_extension_api::{self as zed, http_client::RedirectPolicy, Result};
use serde::Deserialize;

struct RagExtension;

#[derive(Deserialize, Debug)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct Message {
    content: String,
}

impl zed::Extension for RagExtension {
    fn new() -> Self {
        Self
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        arguments: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput> {
        if command.name != "rag" {
            return Err("Invalid command. Expected 'rag'.".into());
        }

        let query = arguments.join(" ");
        if query.is_empty() {
            return Ok(zed::SlashCommandOutput {
                text: "Error: Query not provided. Please enter a prompt.".to_string(),
                sections: vec![],
            });
        }

        let request = HttpRequest {
            method: HttpMethod::Post,
            url: "http://127.0.0.1:8000/v1/chat/completions".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(
                serde_json::to_vec(&json!({
                    "model": "openai:gpt-4o",
                    "messages": [
                        { "role": "system", "content": "You are a helpful consultant." },
                        { "role": "user", "content": query }
                    ],
                    "temperature": 0.1
                }))
                .map_err(|err| format!("Failed to serialize request body: {}", err))?,
            ),
            redirect_policy: RedirectPolicy::FollowAll,
        };

        let mut stream = zed::http_client::fetch_stream(&request)
            .map_err(|err| format!("HTTP request failed: {}", err))?;

        let mut sections = Vec::new();
        loop {
			match stream.next_chunk().map_err(|err| format!("Stream error: {}", err))? {
				Some(response_chunk) => {
					let response_body = String::from_utf8(response_chunk)
						.map_err(|err| format!("Failed to parse response chunk: {}", err))?;

					let api_response: ApiResponse = serde_json::from_str(&response_body)
						.map_err(|err| format!("Failed to deserialize API response: {}", err))?;

					for choice in api_response.choices {
						sections.push(zed::SlashCommandOutputSection {
							range: (0..choice.message.content.len()).into(),
							label: choice.message.content,
						});
					}
				}
				None => break, // End of stream
			}
		}
		
        Ok(zed::SlashCommandOutput {
            text: "RAG API Response".to_string(),
            sections,
        })
    }
}

zed::register_extension!(RagExtension);

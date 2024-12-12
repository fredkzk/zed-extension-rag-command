use zed::{
    http_client::{HttpMethod, HttpRequest},
    serde_json::{self, json},
};
use zed_extension_api::{self as zed, http_client::RedirectPolicy, Result};
use serde::Deserialize;

struct RagExtension;

#[derive(Deserialize, Debug)]
struct RagSearchResult {
    data: String, // Matches aichat response's "data" field
}

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

        // Step 1: Query the RAG search API
        let search_request = HttpRequest {
            method: HttpMethod::Post,
            url: "http://127.0.0.1:8000/v1/rags/search".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(
                serde_json::to_vec(&json!({
                    "name": "deno2",  // Matches the "name" field in the aichat payload
                    "input": query,  // Matches the "input" field
                }))
                .map_err(|err| format!("Failed to serialize search request: {}", err))?,
            ),
            redirect_policy: RedirectPolicy::FollowAll,
        };

        let search_response = zed::http_client::fetch(&search_request)
            .map_err(|err| format!("RAG search request failed: {}", err))?;

        let search_body = String::from_utf8(search_response.body)
            .map_err(|err| format!("Failed to parse RAG search response: {}", err))?;

        let rag_search: RagSearchResult = serde_json::from_str(&search_body)
            .map_err(|err| format!("Failed to deserialize RAG search response: {}", err))?;

        // Step 2: Build the chat completion prompt
        let system_content = format!(
            "You are a helpful assistant, expert in Deno, Fresh, and TypeScript. Use the following context to answer:\n\n{}",
            rag_search.data
        );

        let completion_request = HttpRequest {
            method: HttpMethod::Post,
            url: "http://127.0.0.1:8000/v1/chat/completions".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(
                serde_json::to_vec(&json!({
                    "model": "openai:gpt-4o",
                    "messages": [
                        { "role": "system", "content": system_content },
                        { "role": "user", "content": query }
                    ],
                    "temperature": 0.1
                }))
                .map_err(|err| format!("Failed to serialize completion request: {}", err))?,
            ),
            redirect_policy: RedirectPolicy::FollowAll,
        };

        let mut stream = zed::http_client::fetch_stream(&completion_request)
            .map_err(|err| format!("Chat completion request failed: {}", err))?;

        // Step 3: Process streaming response
        let mut response_text = String::new();
        while let Some(response_chunk) = stream.next_chunk().map_err(|err| format!("Stream error: {}", err))? {
            let response_body = String::from_utf8(response_chunk)
                .map_err(|err| format!("Failed to parse response chunk: {}", err))?;

            let api_response: ApiResponse = serde_json::from_str(&response_body)
                .map_err(|err| format!("Failed to deserialize API response: {}", err))?;

            for choice in api_response.choices {
                response_text.push_str(&choice.message.content);
            }
        }

        // Step 4: Return the assistant response
        Ok(zed::SlashCommandOutput {
            text: response_text.trim().to_string(), // Direct assistant response
            sections: vec![],                      // Optional, add relevant sections if needed
        })
    }
}

zed::register_extension!(RagExtension);

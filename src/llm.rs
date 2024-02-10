use crate::errors::SummaryError;
use crate::release::get_release_notes;
use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};
use async_openai::Client;
use octocrab::models::repos::Release;

/// The GPT-4 model name.
///
/// TODO: Make this configurable via CLI flag or environment variable.
const GPT4: &str = "gpt-4";

/// Creates the summary for the release notes by querying the LLM.
pub async fn create_summary(release: &Release) -> Result<String, SummaryError> {
    let notes = get_release_notes(release)?;
    let summary_prompt = build_summary_prompt(notes.as_str());
    prompt_for_summary(summary_prompt).await
}

/// Builds the prompt for the LLM to generate the release notes summary.
fn build_summary_prompt(release_notes: &str) -> String {
    format!(
        "Please provide a brief summary for the following release notes using bullet points.\
         You do not need to mention the version or release data, only the changes.\
         Please also just provide a description of the changes but don't mention the change types like State Machine Breaking.\
         Please do not include any pull request links.\
         Please keep the summary to a maximum of 10 bullet points.\
         \n\"{}\"}}",
        release_notes
    )
}

/// Prompts the LLM to get the summary for the release notes.
async fn prompt_for_summary(prompt: String) -> Result<String, SummaryError> {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2000u16)
        .model(GPT4)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into()])
        .build()?;

    let response = client.chat().create(request).await?;
    let choice = response.choices.first().ok_or(SummaryError::NoSummary)?;
    let summary = choice
        .message
        .content
        .clone()
        .ok_or(SummaryError::NoSummary)?;

    Ok(summary)
}

#[cfg(test)]
mod summary_tests {
    use super::*;
    use octocrab::models::repos::Release;

    #[tokio::test]
    async fn test_create_summary() {
        let release: Release = serde_json::from_str(include_str!("testdata/release.json"))
            .expect("failed to parse release JSON");

        let res = create_summary(&release).await;
        assert!(
            res.is_ok(),
            "expected no error; got:\n{}\n",
            res.unwrap_err()
        );

        let summary = res.unwrap();
        assert_ne!(
            summary,
            release.body.unwrap(),
            "expected summary to be different from raw release notes"
        );
    }
}

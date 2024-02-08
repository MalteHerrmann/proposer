use octocrab::models::repos::Release;
use crate::errors::SummaryError;
use crate::release::get_release_notes;

/// Creates the summary for the release notes by querying the LLM.
pub fn create_summary(release: &Release) -> Result<String, SummaryError> {
    let notes = get_release_notes(release)?;
    let summary_prompt = build_summary_prompt(notes.as_str());
    prompt_for_summary(summary_prompt)
}

/// Builds the prompt for the LLM to generate the release notes summary.
fn build_summary_prompt(release_notes: &str) -> String {
    format!(
        "Please provide a summary for the following release notes:\n\"{}\"}}",
        release_notes
    )
}

/// Prompts the LLM to get the summary for the release notes.
fn prompt_for_summary(prompt: String) -> Result<String, SummaryError> {
    // TODO: prompt the LLM using the async-openai crate
    println!("Prompting the LLM: {}", prompt);

    Ok("This is a summary of the release notes.".to_string())
}

#[cfg(test)]
mod summary_tests {
    use octocrab::models::repos::Release;
    use super::*;

    #[test]
    fn test_create_summary() {
        let release: Release = serde_json::from_str(include_str!("testdata/release.json"))
            .expect("failed to parse release JSON");

        let res = create_summary(&release);
        assert!(res.is_ok());

        let summary = res.unwrap();
        assert_ne!(summary, release.body.unwrap(), "expected summary to be different from raw release notes");
    }
}

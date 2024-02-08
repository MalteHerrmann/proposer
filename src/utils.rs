/// Wrapper method to write the proposal contents to a file.
pub fn write_content_to_file(
    proposal: &str,
    proposal_file_name: &str,
) -> Result<(), std::io::Error> {
    std::fs::write(proposal_file_name, proposal)
}

#[cfg(test)]
mod tests {
    use crate::network::Network;
    use crate::utils;

    #[test]
    fn test_write_proposal_to_file_pass() {
        let proposal_file_name = format!("proposal-{}-{}.md", Network::Mainnet, "v0.1.0");
        let result = utils::write_content_to_file("test", proposal_file_name.as_str());
        assert!(
            result.is_ok(),
            "Error writing proposal to file: {}",
            result.unwrap_err(),
        );

        // Check that file exists
        assert!(
            std::path::Path::new(proposal_file_name.as_str()).exists(),
            "Proposal file does not exist",
        );

        // Clean up
        std::fs::remove_file(proposal_file_name).unwrap();
    }
}

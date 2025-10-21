use crate::{
    block::N_BLOCKS, errors::ProposalError, helper::UpgradeHelper, inputs::get_time_string,
};
use handlebars::Handlebars;
use num_format::ToFormattedString;
use serde_json::json;

/// Renders the proposal template, filling in the necessary information.
///
/// TODO: this can be removed or moved to another tool / maybe a plugin type thing
pub fn render_proposal(helper: &UpgradeHelper) -> Result<String, ProposalError> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    handlebars
        .register_template_file("proposal", "src/templates/proposal.hbs")
        .unwrap();

    let height_link = format!(
        "[{}](https://mintscan.io/evmos/blocks/{})",
        helper.upgrade_config.upgrade_height, helper.upgrade_config.upgrade_height
    );
    let n_blocks = N_BLOCKS.to_formatted_string(&num_format::Locale::en);
    let upgrade_time = get_time_string(helper.upgrade_config.upgrade_time);

    let data = json!({
        "author": "Malte Herrmann, Evmos Core Team",
        "diff_link": format!("https://github.com/evmos/evmos/compare/{}..{}",
            helper.upgrade_config.previous_version,
            helper.upgrade_config.target_version,
        ),
        "estimated_time": upgrade_time,
        "features": helper.upgrade_config.summary,
        "height": height_link,
        "name": helper.upgrade_config.upgrade_name,
        "n_blocks": n_blocks,
        "network": helper.network_config.name,
        "previous_version": get_release_md_link(helper.upgrade_config.previous_version.as_str()),
        "version": get_release_md_link(helper.upgrade_config.target_version.as_str()),
        "voting_time": helper.network_config.voting_period,
    });

    Ok(handlebars.render("proposal", &data)?)
}

/// Returns the appropriate Markdown link to the release on GitHub for the given version.
fn get_release_md_link(version: &str) -> String {
    format!(
        "[{0}](https://github.com/evmos/evmos/releases/tag/{0})",
        version
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::config::{NetworkConfig, UpgradeConfig};

    #[test]
    fn test_render_proposal_pass() {
        let nc = NetworkConfig::default();
        let uc = UpgradeConfig {
            previous_version: "v0.0.1".to_string(),
            target_version: "v0.1.0".to_string(),
            upgrade_time: Utc::now(),
            upgrade_height: 60,
            ..UpgradeConfig::default()
        };

        let helper = UpgradeHelper::new(&nc, &uc);

        let result = render_proposal(&helper);
        assert!(
            result.is_ok(),
            "Error rendering proposal: {}",
            result.unwrap_err(),
        );
    }
}

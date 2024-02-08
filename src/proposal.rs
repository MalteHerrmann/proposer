use crate::{
    block::N_BLOCKS, errors::ProposalError, helper::UpgradeHelper, inputs::get_time_string,
    network::Network,
};
use handlebars::Handlebars;
use num_format::ToFormattedString;
use serde_json::json;

/// Renders the proposal template, filling in the necessary information.
pub fn render_proposal(helper: &UpgradeHelper) -> Result<String, ProposalError> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    handlebars
        .register_template_file("proposal", "src/templates/proposal.hbs")
        .unwrap();

    let height_link = get_height_with_link(helper.network, helper.upgrade_height);
    let n_blocks = N_BLOCKS.to_formatted_string(&num_format::Locale::en);
    let upgrade_time = get_time_string(helper.upgrade_time);

    let data = json!({
        "author": "Malte Herrmann, Evmos Core Team",
        "diff_link": format!("https://github.com/evmos/evmos/compare/{}..{}",
            helper.previous_version,
            helper.target_version,
        ),
        "estimated_time": upgrade_time,
        "features": helper.summary,
        "height": height_link,
        "name": helper.proposal_name,
        "n_blocks": n_blocks,
        "network": format!("{}", helper.network), // TODO: implement serialize trait here?
        "previous_version": get_release_md_link(helper.previous_version.as_str()),
        "version": get_release_md_link(helper.target_version.as_str()),
        "voting_time": helper.voting_period,
    });

    Ok(handlebars.render("proposal", &data)?)
}

/// Returns the appropriate Markdown link to the block on Mintscan for the given network and height.
fn get_height_with_link(network: Network, height: u64) -> String {
    let height_with_commas = height.to_formatted_string(&num_format::Locale::en);
    match network {
        Network::LocalNode => format!(
            "[{}](https://www.mintscan.io/evmos/blocks/{})",
            height_with_commas, height
        ),
        Network::Mainnet => format!(
            "[{}](https://www.mintscan.io/evmos/blocks/{})",
            height_with_commas, height
        ),
        Network::Testnet => format!(
            "[{}](https://testnet.mintscan.io/evmos-testnet/blocks/{})",
            height_with_commas, height
        ),
    }
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

    #[test]
    fn test_render_proposal_pass() {
        let helper = UpgradeHelper::new(Network::Mainnet, "v0.0.1", "v0.1.0", Utc::now(), 60, "");

        let result = render_proposal(&helper);
        assert!(
            result.is_ok(),
            "Error rendering proposal: {}",
            result.unwrap_err(),
        );
    }
}

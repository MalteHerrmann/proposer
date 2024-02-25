use crate::errors::CommonwealthError::InvalidCommonwealthLink;
use crate::{errors::InputError, network::Network};
use chrono::{
    DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc, Weekday,
};
use inquire::{validator::Validation::Valid, DateSelect, Select};
use std::{fs, ops::Add, path::PathBuf};

const MONTHS: [&str; 13] = [
    "",
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Scans the current folder for existing proposal configurations (stored as JSON)
/// and lets the user choose the desired configuration file to use.
pub fn choose_config() -> Result<PathBuf, InputError> {
    let current_dir = std::env::current_dir()?;

    // Get all files in the current directory
    let paths = fs::read_dir(&current_dir)?;

    // Filter for JSON files
    let json_files = paths.filter(|path| {
        path.as_ref()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .ends_with(".json")
    });

    // Collect the file names
    let config_files: Vec<String> = json_files
        .map(|file| file.unwrap().path().to_string_lossy().to_string())
        .collect();

    if config_files.is_empty() {
        return Err(InputError::NoConfigFiles(current_dir));
    }

    // Prompt the user to select the configuration file
    //
    // FIXME: Why does the question mark operator not work here? It doesn't register the #[from] attribute in the error enum somehow?
    match Select::new("Select configuration file", config_files).prompt() {
        Ok(file) => Ok(current_dir.join(file)),
        Err(e) => Err(InputError::UserInput(e)),
    }
}

/// Prompts the user to input the link to the Commonwealth proposal and checks if the input is a valid URL
/// that points to the Commonwealth page.
pub async fn choose_commonwealth_link() -> Result<String, InputError> {
    let link = inquire::Text::new("Enter the link to the Commonwealth proposal")
        .with_validator(|input: &str| {
            if input.starts_with("https://commonwealth.im/evmos") {
                Ok(Valid)
            } else {
                // TODO: understand why the into() helps with the Box<dyn Error> here?
                Err(InvalidCommonwealthLink.into())
            }
        })
        .prompt()?;

    Ok(link)
}

/// Prompts the user to select the network type used.
pub fn get_used_network() -> Result<Network, InputError> {
    let network_options = vec!["Local Node", "Testnet", "Mainnet"];
    let chosen_network = Select::new("Select network", network_options).prompt()?;

    // TODO: improve handling here! Should be more elegant to reverse the print stuff from the Network
    // type.
    let used_network = match chosen_network {
        "Local Node" => Network::LocalNode,
        "Testnet" => Network::Testnet,
        "Mainnet" => Network::Mainnet,
        &_ => {
            return Err(InputError::InvalidNetwork(chosen_network.to_string()));
        }
    };

    Ok(used_network)
}

/// Prompts the user to input the duration of the voting period.
/// The duration is given in hours.
pub fn get_evmosd_home(network: &Network) -> Result<PathBuf, InputError> {
    let mut default_path = dirs::home_dir().expect("failed to get home directory");

    match network {
        Network::LocalNode => &default_path.push(".tmp-evmosd"),
        _ => &default_path.push(".evmosd"),
    };

    let selected_option = inquire::Text::new("Enter the home path to your Evmos keyring")
        .with_default(default_path.as_os_str().to_str().unwrap())
        .prompt()?;

    Ok(PathBuf::from(selected_option))
}

/// Prompts the user to input some plain text.
pub fn get_text(prompt: &str) -> Result<String, InputError> {
    Ok(inquire::Text::new(prompt).prompt()?)
}

/// Prompts the user to input the date for the planned upgrade.
/// The date is calculated based on the current time and the voting period duration.
pub fn get_upgrade_time(
    voting_period: Duration,
    utc_time: DateTime<Utc>,
) -> Result<DateTime<Utc>, InputError> {
    let default_date = calculate_planned_date(voting_period, utc_time);

    // Prompt the user to input the desired upgrade date
    let date = DateSelect::new("Select date for the planned upgrade")
        .with_min_date(utc_time.date_naive())
        .with_default(default_date.date_naive())
        .with_week_start(Weekday::Mon)
        .prompt()?;

    let time = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
    let upgrade_time = NaiveDateTime::new(date, time);

    Ok(Utc.from_local_datetime(&upgrade_time).unwrap())
}

/// Calculates the date for the planned upgrade given the current time and the voting period duration.
/// Per default, 4 pm UTC is used as a reference time.
/// If the passed UTC time is after 2 pm UTC, the planned date will be shifted to the next day.
fn calculate_planned_date(voting_period: Duration, utc_time: DateTime<Utc>) -> DateTime<Utc> {
    let mut end_of_voting = utc_time.add(voting_period);

    // NOTE: if using the tool after 2pm UTC or the end of voting would be at or after 2 PM, the upgrade should happen on the next day
    if utc_time.hour() > 14 || end_of_voting.hour() >= 16 {
        end_of_voting = end_of_voting.add(Duration::days(1));
    }

    // NOTE: we don't want to upgrade on a weekend, so we shift the upgrade to the next monday
    if end_of_voting.weekday() == Weekday::Sat {
        end_of_voting = end_of_voting.add(Duration::days(2));
    } else if end_of_voting.weekday() == Weekday::Sun {
        end_of_voting = end_of_voting.add(Duration::days(1));
    }

    Utc.with_ymd_and_hms(
        end_of_voting.year(),
        end_of_voting.month(),
        end_of_voting.day(),
        16,
        0,
        0,
    )
    .unwrap()
}

/// Checks if the passed upgrade time is valid.
/// The upgrade time cannot be on a weekend.
pub fn is_valid_upgrade_time(upgrade_time: DateTime<Utc>) -> bool {
    if upgrade_time.weekday() == Weekday::Sat || upgrade_time.weekday() == Weekday::Sun {
        return false;
    }

    true
}

/// Returns a string representation of the upgrade time.
pub fn get_time_string(time: DateTime<Utc>) -> String {
    let (is_pm, hour) = time.hour12();
    format!(
        "{}{} {} on {}., {} {}., {}",
        hour,
        if is_pm { "PM" } else { "AM" },
        time.timezone(),
        time.weekday(),
        MONTHS[time.month() as usize],
        time.day(),
        time.year(),
    )
}

/// Lets the user choose the desired key to use.
pub fn get_key(keys: Vec<String>) -> Result<String, InputError> {
    Ok(Select::new("Select key to submit proposal", keys).prompt()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, Utc};
    use rstest::{fixture, rstest};

    #[fixture]
    fn monday_morning() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 10, 23, 11, 0, 0).unwrap()
    }

    #[fixture]
    fn monday_evening() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 10, 23, 20, 0, 0).unwrap()
    }

    #[fixture]
    fn friday_morning() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 10, 27, 11, 0, 0).unwrap()
    }

    #[fixture]
    fn testnet_voting_period() -> Duration {
        Duration::hours(12)
    }

    #[fixture]
    fn mainnet_voting_period() -> Duration {
        Duration::hours(120)
    }

    #[rstest]
    fn test_calculate_planned_date_monday_morning_testnet(
        monday_morning: DateTime<Utc>,
        testnet_voting_period: Duration,
    ) {
        assert_eq!(
            calculate_planned_date(testnet_voting_period, monday_morning),
            Utc.with_ymd_and_hms(2023, 10, 24, 16, 0, 0).unwrap(),
            "expected different date for testnet upgrade when calling on monday morning",
        );
    }

    #[rstest]
    fn test_calculate_planned_date_monday_morning_mainnet(
        monday_morning: DateTime<Utc>,
        mainnet_voting_period: Duration,
    ) {
        assert_eq!(
            calculate_planned_date(mainnet_voting_period, monday_morning),
            // NOTE: the upgrade should happen on the next monday 4PM, not on saturday which would be t+120h
            Utc.with_ymd_and_hms(2023, 10, 30, 16, 0, 0).unwrap(),
            "expected different date for mainnet upgrade when calling on monday morning",
        );
    }

    #[rstest]
    fn test_calculate_planned_date_monday_evening_testnet(
        monday_evening: DateTime<Utc>,
        testnet_voting_period: Duration,
    ) {
        assert_eq!(
            calculate_planned_date(testnet_voting_period, monday_evening),
            Utc.with_ymd_and_hms(2023, 10, 25, 16, 0, 0).unwrap(),
            "expected different date for testnet upgrade when calling on monday evening",
        );
    }

    #[rstest]
    fn test_calculate_planned_date_monday_evening_mainnet(
        monday_evening: DateTime<Utc>,
        mainnet_voting_period: Duration,
    ) {
        assert_eq!(
            calculate_planned_date(mainnet_voting_period, monday_evening),
            // NOTE: the upgrade should happen on the next monday 4PM, not on saturday which would be t+120h
            Utc.with_ymd_and_hms(2023, 10, 30, 16, 0, 0).unwrap(),
            "expected different date for mainnet upgrade when calling on monday evening",
        );
    }

    #[rstest]
    fn test_calculate_planned_date_friday_morning_testnet(
        friday_morning: DateTime<Utc>,
        testnet_voting_period: Duration,
    ) {
        assert_eq!(
            calculate_planned_date(testnet_voting_period, friday_morning),
            // NOTE: the upgrade should happen on the next monday 4PM, not on saturday which would be t+12h
            Utc.with_ymd_and_hms(2023, 10, 30, 16, 0, 0).unwrap(),
            "expected different date for testnet upgrade when calling on thursday morning",
        );
    }

    #[rstest]
    fn test_calculate_planned_date_friday_morning_mainnet(
        friday_morning: DateTime<Utc>,
        mainnet_voting_period: Duration,
    ) {
        assert_eq!(
            calculate_planned_date(mainnet_voting_period, friday_morning),
            // NOTE: the upgrade should happen on the next wednesday 4PM
            Utc.with_ymd_and_hms(2023, 11, 1, 16, 0, 0).unwrap(),
            "expected different date for mainnet upgrade when calling on thursday morning",
        );
    }

    #[test]
    fn test_get_time_string_october_morning() {
        let time = Utc.with_ymd_and_hms(2023, 10, 23, 4, 0, 0).unwrap();
        assert_eq!(
            get_time_string(time),
            "4AM UTC on Mon., October 23., 2023",
            "expected different time string",
        );
    }

    #[test]
    fn test_get_time_string_february_evening() {
        let time = Utc.with_ymd_and_hms(2023, 2, 1, 16, 0, 0).unwrap();
        assert_eq!(
            get_time_string(time),
            "4PM UTC on Wed., February 1., 2023",
            "expected different time string",
        );
    }
}

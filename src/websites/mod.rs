use std::{io::Write, time::Duration};

use chrono::{Days, NaiveDate};
use thirtyfour::WebDriver;

mod investing_parser;
mod marketwatch_parser;
mod tradingview_parser;
mod zacks_parser;
mod benzinga_parser;

const MARKETWATCH: &str = "https://www.marketwatch.com/tools/earnings-calendar";
const ZACKS: &str = "https://www.zacks.com/earnings/earnings-calendar?icid=earnings-earnings-nav_tracking-zcom-main_menu_wrapper-earnings_calendar";
const BENZINGA: &str = "https://www.benzinga.com/calendars/earnings";
const INVESTING: &str = "https://www.investing.com/earnings-calendar/";
const EARNINGSWHISPERS: &str = "https://www.earningswhispers.com/calendar";
const TRADINGVIEW: &str =
    "https://www.tradingview.com/markets/stocks-usa/earnings/";
const SCROLL_INTO_VIEW: &str =
    r#"arguments[0].scrollIntoView({behavior: "auto", block: "center"});"#;
const WAIT_INTERVAL: Duration = Duration::from_secs(1);
const LOAD_WAIT: Duration = Duration::from_secs(2);
const TIMEOUT_THREE_SEC: Duration = Duration::from_secs(3);
const TIMEOUT_FIVE_SEC: Duration = Duration::from_secs(5);
const TIMEOUT_TEN_SEC: Duration = Duration::from_secs(10);
const MAX_RERUNS: usize = 1;

#[derive(Debug, Clone)]
pub struct Company {
    pub symbol: String,
    pub name: String,
}

pub async fn marketwatch_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    print!("Reading 'MarketWatch' data...");
    let mut max_reruns = MAX_RERUNS;
    loop {
        std::io::stdout().flush()?;
        match marketwatch_parser::get_data(driver, day).await {
            Ok(c) => {
                println!(" Success!");
                return Ok(c);
            }
            Err(e) => {
                if max_reruns == 0 {
                    println!("\nCouldn't parse data: {e}");
                    return Ok(vec![]);
                }
                print!("Failed to parse data! Trying again...");
                max_reruns -= 1;
                continue;
            }
        }
    }
}

pub async fn zacks_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    print!("Reading 'Zacks' data...");
    let mut max_reruns = MAX_RERUNS;
    loop {
        std::io::stdout().flush()?;
        match zacks_parser::get_data(driver, day).await {
            Ok(c) => {
                println!(" Success!");
                return Ok(c);
            }
            Err(e) => {
                if max_reruns == 0 {
                    println!("\nCouldn't parse data: {e}");
                    return Ok(vec![]);
                }
                print!("Failed to parse data! Trying again...");
                max_reruns -= 1;
                continue;
            }
        }
    }
}

/// Loads maximum results of 150.
pub async fn tradingview_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    print!("Reading 'TradingView' data...");
    let mut max_reruns = 1;
    loop {
        std::io::stdout().flush()?;
        match tradingview_parser::get_data(driver, day).await {
            Ok(c) => {
                println!(" Success!");
                return Ok(c);
            }
            Err(e) => {
                if max_reruns == 0 {
                    println!("\nCouldn't parse data: {e}");
                    return Ok(vec![]);
                }
                print!("Failed to parse data! Trying again...");
                max_reruns -= 1;
                continue;
            }
        }
    }
}

pub async fn investing_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    print!("Reading 'Investing' data...");
    let mut max_reruns = 1;
    loop {
        std::io::stdout().flush()?;
        match investing_parser::get_data(driver, day).await {
            Ok(c) => {
                println!(" Success!");
                return Ok(c);
            }
            Err(e) => {
                if max_reruns == 0 {
                    println!("\nCouldn't parse data: {e}");
                    return Ok(vec![]);
                }
                print!("Failed to parse data! Trying again...");
                max_reruns -= 1;
                continue;
            }
        }
    }
}

pub async fn benzinga_data(driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    print!("Reading 'Benzinga' data...");
    let mut max_reruns = 1;
    loop {
        std::io::stdout().flush()?;
        match benzinga_parser::get_data(driver, day).await {
            Ok(c) => {
                println!(" Success!");
                return Ok(c);
            }
            Err(e) => {
                if max_reruns == 0 {
                    println!("\nCouldn't parse data: {e}");
                    return Ok(vec![]);
                }
                print!("Failed to parse data! Trying again...");
                max_reruns -= 1;
                continue;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RelativeDay {
    Yesterday,
    Today,
    Tomorrow,
}

impl RelativeDay {
    pub fn get_date(&self) -> NaiveDate {
        use chrono::offset::Local;
        let now = Local::now();
        match self {
            RelativeDay::Yesterday => {
                now.checked_sub_days(Days::new(1)).unwrap().date_naive()
            }
            RelativeDay::Today => now.date_naive(),
            RelativeDay::Tomorrow => {
                now.checked_add_days(Days::new(1)).unwrap().date_naive()
            }
        }
    }
}

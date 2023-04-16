use chrono::{Days, NaiveDate};
use thirtyfour::WebDriver;

mod marketwatch;
mod zacks;

const MARKETWATCH: &str = "https://www.marketwatch.com/tools/earnings-calendar";
const ZACKS: &str = "https://www.zacks.com/earnings/earnings-calendar?icid=earnings-earnings-nav_tracking-zcom-main_menu_wrapper-earnings_calendar";
const BENZINGA: &str = "https://www.benzinga.com/calendars/earnings";
const INVESTING: &str = "https://www.investing.com/earnings-calendar/";
const CNBC: &str = "https://www.cnbc.com/earnings-calendar/";
const EARNINGS_WHISPERS: &str = "https://www.earningswhispers.com/calendar";
const TRADING_VIEW: &str =
    "https://www.tradingview.com/markets/stocks-usa/earnings/";

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
    let mut max_reruns = 1;
    loop {
        match marketwatch::get_marketwatch_data(driver, day).await {
            Ok(c) => {
                println!("Success!");
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
    let data = zacks::get_zacks_data(driver, day).await.unwrap();
    println!("Success!");
    Ok(data)
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

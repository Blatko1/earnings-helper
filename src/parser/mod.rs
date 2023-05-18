mod benzinga_parser;
mod investing_parser;
mod marketwatch_parser;
mod tradingview_parser;
mod zacks_parser;

use async_trait::async_trait;
use std::{io::Write, time::Duration};

use chrono::{Days, NaiveDate};
use thirtyfour::{DesiredCapabilities, WebDriver};

use self::{
    benzinga_parser::BenzingaParser, investing_parser::InvestingParser,
    marketwatch_parser::MarketWatchParser,
    tradingview_parser::TradingViewParser, zacks_parser::ZacksParser,
};

const MARKETWATCH: &str = "https://www.marketwatch.com/tools/earnings-calendar";
const ZACKS: &str = "https://www.zacks.com/earnings/earnings-calendar?icid=earnings-earnings-nav_tracking-zcom-main_menu_wrapper-earnings_calendar";
const BENZINGA: &str = "https://www.benzinga.com/calendars/earnings";
const INVESTING: &str = "https://www.investing.com/earnings-calendar/";
// const EARNINGSWHISPERS: &str = "https://www.earningswhispers.com/calendar";
const TRADINGVIEW: &str =
    "https://www.tradingview.com/markets/stocks-usa/earnings/";

const SCROLL_INTO_VIEW: &str =
    r#"arguments[0].scrollIntoView({behavior: "instant", block: "center"});"#;
const WAIT_INTERVAL: Duration = Duration::from_millis(400);
const LOAD_WAIT_SHORT: Duration = Duration::from_secs(1);
const LOAD_WAIT: Duration = Duration::from_secs(2);
const TIMEOUT_THREE_SEC: Duration = Duration::from_secs(3);
const TIMEOUT_FIVE_SEC: Duration = Duration::from_secs(5);
const TIMEOUT_TEN_SEC: Duration = Duration::from_secs(10);
const MAX_RERUNS: usize = 1;

/// Returns all parsed data in one `Vec` with an average of entries
/// per parsed website.
pub async fn parse_website_data() -> anyhow::Result<(Vec<Company>, usize)> {
    let mut stdout = std::io::stdout().lock();
    write!(stdout, "Initializing WebDriver...")?;
    stdout.flush()?;

    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::new("http://localhost:9515", caps)
        .await
        .map_err(|e| writeln!(stdout, "Is chromedriver started? Error: {e}"))
        .unwrap();
    writeln!(stdout, "Success!")?;

    let day = RelativeDay::Tomorrow;

    let web_parsers = vec![
        Parser::Marketwatch,
        Parser::Zacks,
        Parser::Tradingview,
        Parser::Investing,
        Parser::Benzinga,
    ];
    let parsed = parse_all(&driver, day, web_parsers).await?;

    driver.quit().await?;

    let parsed_websites = parsed.len();
    // Store all data into one big array.
    let data: Vec<Company> = parsed.into_iter().flatten().collect();
    let avg = parsed_websites / data.len();
    Ok((data, avg))
}

async fn parse_all(
    driver: &WebDriver,
    day: RelativeDay,
    web_parsers: Vec<Parser>,
) -> anyhow::Result<Vec<Vec<Company>>> {
    let mut stdout = std::io::stdout().lock();
    let mut result = Vec::with_capacity(web_parsers.len());

    for parser in web_parsers {
        write!(stdout, "Reading '{}' data...", parser.get_name())?;
        let mut max_reruns = MAX_RERUNS;

        let parsed = loop {
            stdout.flush()?;

            match parser.parse(driver, day).await {
                Ok(c) => {
                    writeln!(stdout, " Success!")?;
                    break c;
                }
                Err(e) => {
                    if max_reruns == 0 {
                        writeln!(stdout, "\nCouldn't parse data: {e}")?;
                        break vec![];
                    }
                    writeln!(stdout, "Failed to parse data: {e}")?;
                    write!(stdout, "Trying again...")?;
                    max_reruns -= 1;
                    continue;
                }
            }
        };

        result.push(parsed);
    }
    Ok(result)
}

enum Parser {
    Marketwatch,
    Zacks,
    Tradingview,
    Investing,
    Benzinga,
}

impl Parser {
    fn get_name(&self) -> &'static str {
        match self {
            Self::Marketwatch => MarketWatchParser::NAME,
            Self::Zacks => ZacksParser::NAME,
            Self::Tradingview => TradingViewParser::NAME,
            Self::Investing => InvestingParser::NAME,
            Self::Benzinga => BenzingaParser::NAME,
        }
    }

    async fn parse(
        &self,
        driver: &WebDriver,
        day: RelativeDay,
    ) -> anyhow::Result<Vec<Company>> {
        match self {
            Self::Marketwatch => MarketWatchParser::parse(driver, day).await,
            Self::Zacks => ZacksParser::parse(driver, day).await,
            Self::Tradingview => TradingViewParser::parse(driver, day).await,
            Self::Investing => InvestingParser::parse(driver, day).await,
            Self::Benzinga => BenzingaParser::parse(driver, day).await,
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Company {
    pub symbol: String,
    pub name: String,
}

impl PartialEq for Company {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }
}

impl PartialOrd for Company {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.symbol.partial_cmp(&other.symbol)
    }
}

impl Ord for Company {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.symbol.cmp(&other.symbol)
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

#[async_trait]
trait WebsiteParser {
    const NAME: &'static str;

    async fn parse(
        driver: &WebDriver,
        day: RelativeDay,
    ) -> anyhow::Result<Vec<Company>>;
}

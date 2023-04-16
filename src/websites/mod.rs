pub mod marketwatch;
pub mod zacks;

const MARKETWATCH: &str = "https://www.marketwatch.com/tools/earnings-calendar";
const ZACKS: &str = "https://www.zacks.com/earnings/earnings-calendar?icid=earnings-earnings-nav_tracking-zcom-main_menu_wrapper-earnings_calendar";
const BENZINGA: &str = "https://www.benzinga.com/calendars/earnings";
const INVESTING: &str = "https://www.investing.com/earnings-calendar/";
const CNBC: &str = "https://www.cnbc.com/earnings-calendar/";
const EARNINGS_WHISPERS: &str = "https://www.earningswhispers.com/calendar";
const TRADING_VIEW: &str =
    "https://www.tradingview.com/markets/stocks-usa/earnings/";

#[derive(Debug, Clone, Default)]
pub struct Company {
    pub symbol: String,
    pub name: String,
}

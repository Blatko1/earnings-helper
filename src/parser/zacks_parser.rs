use async_trait::async_trait;
use chrono::{Datelike, Days, NaiveDate, Weekday};
use std::vec;
use thirtyfour::{prelude::ElementQueryable, By, WebDriver};

use super::{
    Company, WebsiteParser, SCROLL_INTO_VIEW, TIMEOUT_FIVE_SEC,
    TIMEOUT_TEN_SEC, WAIT_INTERVAL, ZACKS,
};
use crate::{parser::LOAD_WAIT_SHORT, RelativeDay};

const PREVIOUS_WEEK_SELECTOR: &str = "div[class=\"prenext_txt align_left\"]>a";
const NEXT_WEEK_SELECTOR: &str = "div[class=\"prenext_txt align_right\"]>a";
const SHOW_ENTRIES_SELECTOR: &str =
    "div#earnings_rel_data_all_table_length>label>select";
const SYMBOL_SELECTOR: &str =
    "table#earnings_rel_data_all_table>tbody>tr>th>a>span";
const COMPANY_NAME_SELECTOR: &str =
    "table#earnings_rel_data_all_table>tbody>tr>td:nth-child(2)>span";
const SHOW_ALL_BUTTON_SELECTOR: &str = "option[value=\"-1\"]";
const ACCEPT_BUTTON_CSS: &str =
    "button[class=\"Button__StyledButton-a1qza5-0 fLZgds\"]";
const COOKIE_ACCEPT_CSS: &str = "button[id=\"accept_cookie\"";
const EVENTS_TITLE: &str = "WeeklyEventsTitle";
const READ_MODE_BUTTON_CSS: &str = "button[class=\"Button__StyledButton-a1qza5-0 fLZgds\"]";

pub struct ZacksParser {}

#[async_trait]
impl WebsiteParser for ZacksParser {
    const NAME: &'static str = "Zacks";

    async fn parse(
        driver: &WebDriver,
        day: RelativeDay,
    ) -> anyhow::Result<Vec<Company>> {
        driver.goto(ZACKS).await?;
        // Accept cookies in order to remove the cookies 'obstacle' dialog box.
        accept_cookies(driver).await.unwrap_or(());

        let today = chrono::offset::Local::now().date_naive();
        let target = day.get_date();
        let target_weekday = target.weekday();
        // Weekdays here are counted from Sun to Sat.
        match today.weekday() {
            // Check if the target week is before the today week
            Weekday::Sun => {
                if target_weekday == Weekday::Sat {
                    to_previous_week(driver).await?;
                }
            }
            // Check if the target week is after the today week
            Weekday::Sat => {
                if target_weekday == Weekday::Sun {
                    to_next_week(driver).await?;
                }
            }
            _ => (),
        }
        parse_data(driver, target).await
    }
}

async fn to_previous_week(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(PREVIOUS_WEEK_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'Previous Week' button")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
        .await?;
    button.click().await?;
    Ok(())
}

async fn to_next_week(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(NEXT_WEEK_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'Next Week' button")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
        .await?;
    button.click().await?;
    Ok(())
}

async fn accept_cookies(driver: &WebDriver) -> anyhow::Result<()> {
    if let Ok(button) = driver.query(By::Css(READ_MODE_BUTTON_CSS))
    .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
    .desc("Find the 'Read more' button")
    .single()
        .await {
            button.click().await?;
        }
    driver
        .query(By::Css(ACCEPT_BUTTON_CSS))
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .desc("Find the 'Accept' button")
        .single()
        .await?
        .click()
        .await?;
    driver
        .query(By::Css(COOKIE_ACCEPT_CSS))
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .desc("Find the 'Accept cookies' button")
        .single()
        .await?
        .click()
        .await?;
    Ok(())
}

// TODO branch this function into smaller functions.
async fn parse_data(
    driver: &WebDriver,
    date: NaiveDate,
) -> anyhow::Result<Vec<Company>> {
    // Calculate and create needed header IDs.
    let weekday = date.weekday();
    let weekday_num = weekday.num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    let header_text = format!(
        "Events For {} - {}",
        week_start.format("%-m/%-d/%Y"),
        week_end.format("%-m/%-d/%Y")
    );
    let header_id = format!("d_{}", weekday_num);
    let earnings_link_id = format!("cal_link_{}", weekday_num);
    let earnings_selector_css =
        format!("a#{}[evt_type=\"1\"]", earnings_link_id);
    

    driver
        .query(By::Id(EVENTS_TITLE))
        .with_text(header_text)
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .single()
        .await?;
    if let Ok(e) = driver
        .query(By::Css(&earnings_selector_css))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find the 'earnings' button")
        .single()
        .await
    {
        loop {
            if e.click().await.is_err() {
                driver
                    .query(By::Id(&header_id))
                    .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
                    .desc("Click date header to access data")
                    .single()
                    .await?
                    .click()
                    .await?;
                continue;
            }
            break;
        }
    } else {
        return Ok(vec![]);
    }
    tokio::time::sleep(LOAD_WAIT_SHORT).await;
    let selector = driver
        .query(By::Css(SHOW_ENTRIES_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find entries selector")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![selector.to_json()?])
        .await?;
    selector.click().await?;
    driver
        .query(By::Css(SHOW_ALL_BUTTON_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Click on the 'show all' button")
        .single()
        .await?
        .click()
        .await?;

    let source = driver.source().await?;
    let document = scraper::Html::parse_document(&source);
    let symbol_selector = scraper::Selector::parse(SYMBOL_SELECTOR)
        .map_err(|e| eprintln!("{e}"))
        .unwrap();
    let names_selector = scraper::Selector::parse(COMPANY_NAME_SELECTOR)
        .map_err(|e| eprintln!("{e}"))
        .unwrap();
    let mut symbols: Vec<String> = document
        .select(&symbol_selector)
        .map(|e| e.inner_html())
        .collect();
    let names: Vec<String> = document
        .select(&names_selector)
        .map(|e| e.inner_html())
        .collect();

    for sym in symbols.iter_mut() {
        *sym = sym.split('<').collect::<Vec<&str>>()[0].to_string();
    }
    assert_eq!(symbols.len(), names.len());

    let companies: Vec<Company> = symbols
        .iter()
        .zip(names.iter())
        .map(|(s, n)| Company {
            symbol: s.clone(),
            name: n.clone(),
        })
        .collect();

    Ok(companies)
}

#[test]
fn week_range_dates() {
    let date = NaiveDate::from_ymd_opt(2023, 4, 17).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/16/2023 - 4/22/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 16).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/16/2023 - 4/22/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 15).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/9/2023 - 4/15/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 6).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/2/2023 - 4/8/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 24).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/23/2023 - 4/29/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 30).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/30/2023 - 5/6/2023"
    );
}

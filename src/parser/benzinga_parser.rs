use async_trait::async_trait;
use chrono::Datelike;
use thirtyfour::{
    prelude::{ElementQueryable, ElementWaitable},
    By, WebDriver,
};

use super::{
    Company, WebsiteParser, BENZINGA, LOAD_WAIT_SHORT, SCROLL_INTO_VIEW,
    TIMEOUT_FIVE_SEC, TIMEOUT_TEN_SEC, WAIT_INTERVAL,
};
use crate::RelativeDay;

const POPUP_CLOSE_BUTTON1_SELECTOR: &str =
    "button[class=\"StyledIconButtonElement-sc-114x20d-0 jdccT prosper-field-button\"";
const POPUP_CLOSE_BUTTON2_SELECTOR: &str =
    "armington-ButtonElement--KAlieQqWWrwt2Isu6MDd";
const PREVIOUS_MONTH_BUTTON_SELECTOR: &str =
    "span[class=\"DayPicker-NavButton DayPicker-NavButton--prev\"]";
const DATE_PICKER_SELECTOR: &str =
    "div[class=\"range-date-picker__field-wrapper\"]";
const SYMBOL_SELECTOR: &str =
    "tr[class=\"ant-table-row ant-table-row-level-0\"]>td:nth-child(3)>div>div>div>a";

pub struct BenzingaParser {}

#[async_trait]
impl WebsiteParser for BenzingaParser {
    const NAME: &'static str = "Benzinga";

    async fn parse(
        driver: &WebDriver,
        day: RelativeDay,
    ) -> anyhow::Result<Vec<Company>> {
        driver.goto(BENZINGA).await?;
        loop {
            if pick_date(driver, day).await.is_err() {
                close_popup(driver).await?;
                continue;
            }
            break;
        }
        // Wait for the results to load
        tokio::time::sleep(LOAD_WAIT_SHORT).await;

        parse_data(driver).await
    }
}

async fn pick_date(
    driver: &WebDriver,
    target: RelativeDay,
) -> anyhow::Result<()> {
    let calendar = driver
        .query(By::Css(DATE_PICKER_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Click on the calendar")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![calendar.to_json()?])
        .await?;
    calendar.click().await?;

    let today = chrono::Local::now().date_naive();
    let target_date = target.get_date();
    if today.month() != target_date.month() {
        if let RelativeDay::Yesterday = target {
            to_previous_month(driver).await?;
        }
    }
    let formatted_date = target_date.format("%a %b %d %Y").to_string();
    let day_picker_selector =
        &format!("div[aria-label=\"{}\"]", formatted_date);
    let target_date_button = driver
        .query(By::Css(day_picker_selector))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find the 'day picker' for target date")
        .single()
        .await?;

    // Click two times to only show results for the target date.
    target_date_button.click().await?;
    target_date_button.click().await?;
    Ok(())
}

async fn to_previous_month(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(PREVIOUS_MONTH_BUTTON_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'prev month' button")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
        .await?;
    button.click().await?;
    Ok(())
}

async fn close_popup(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(POPUP_CLOSE_BUTTON1_SELECTOR))
        .or(By::Id(POPUP_CLOSE_BUTTON2_SELECTOR))
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .desc("Find popup close button")
        .single()
        .await?;
    button
        .wait_until()
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .displayed()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
        .await?;
    button.click().await?;
    Ok(())
}

async fn parse_data(driver: &WebDriver) -> anyhow::Result<Vec<Company>> {
    let source = driver.source().await?;
    let document = scraper::Html::parse_document(&source);
    let symbol_selector = scraper::Selector::parse(SYMBOL_SELECTOR)
        .map_err(|e| eprintln!("{e}"))
        .unwrap();
    let symbols: Vec<String> = document
        .select(&symbol_selector)
        .map(|e| e.inner_html())
        .collect();

    let companies: Vec<Company> = symbols
        .iter()
        .map(|s| Company {
            symbol: s.clone(),
            name: String::new(),
        })
        .collect();
    Ok(companies)
}

#[test]
fn date_weekday_month_day_year() {
    let date = chrono::NaiveDate::from_ymd_opt(2023, 4, 12).unwrap();
    let formatted = date.format("%a %b %d %Y").to_string();
    assert_eq!(formatted, "Wed Apr 12 2023");

    let date = chrono::NaiveDate::from_ymd_opt(2023, 4, 14).unwrap();
    let formatted = date.format("%a %b %d %Y").to_string();
    assert_eq!(formatted, "Fri Apr 14 2023");
}

use std::{time::Duration, vec};
use thirtyfour::{prelude::ElementQueryable, By, WebDriver};

use super::{Company, INVESTING};
use crate::RelativeDay;

const COOKIE_ACCEPT_ID: &str = "onetrust-accept-btn-handler";
const POPUP_CLOSE_BUTTON_SELECTOR: &str = "i[class=\"popupCloseIcon largeBannerCloser\"]";
const PREVIOUS_DAY_ID: &str = "timeFrame_yesterday";
const SYMBOL_SELECTOR: &str = "a[class=\"bold middle\"]";
const COMPANY_NAME_SELECTOR: &str = "span[class=\"earnCalCompanyName middle\"]";
const TODAY_DAY_ID: &str = "timeFrame_today";
const NEXT_DAY_ID: &str = "timeFrame_tomorrow";
const WAIT_INTERVAL: Duration = Duration::from_secs(1);
const LOAD_WAIT: Duration = Duration::from_secs(2);
const TIMEOUT_FIVE_SEC: Duration = Duration::from_secs(5);
const SCROLL_INTO_VIEW: &str =
    r#"arguments[0].scrollIntoView({behavior: "auto", block: "center"});"#;

pub async fn get_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    driver.goto(INVESTING).await?;
    // Accept cookies in order to remove the cookies 'obstacle' dialog box.
    accept_cookies(driver).await?;

    // Wait for the popup to show (sometimes)
    tokio::time::sleep(LOAD_WAIT).await;
    close_popup(driver).await.unwrap_or_else(|_|{});

    match day {
        RelativeDay::Yesterday => to_previous_day(driver).await?,
        RelativeDay::Today => to_today_day(driver).await?,
        RelativeDay::Tomorrow => to_next_day(driver).await?,
    }
    // Wait for the browser to load data table
    tokio::time::sleep(LOAD_WAIT).await;

    parse_data(driver).await
}

async fn to_previous_day(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Id(PREVIOUS_DAY_ID))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'Yesterday' button")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
        .await?;
    button.click().await?;
    Ok(())
}

async fn to_today_day(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Id(TODAY_DAY_ID))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'Today' button")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
        .await?;
    button.click().await?;
    Ok(())
}

async fn to_next_day(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Id(NEXT_DAY_ID))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'Tomorrow' button")
        .single()
        .await?;
    driver
        .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
        .await?;
    button.click().await?;
    Ok(())
}
// TODO Check and evaluate all unwraps
async fn accept_cookies(driver: &WebDriver) -> anyhow::Result<()> {
    driver
        .query(By::Id(COOKIE_ACCEPT_ID))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find cookie accept button")
        .single()
        .await?
        .click()
        .await?;
    Ok(())
}

async fn close_popup(driver: &WebDriver) -> anyhow::Result<()> {
    driver
        .query(By::Css(POPUP_CLOSE_BUTTON_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find popup close button")
        .single()
        .await?
        .click()
        .await?;
    Ok(())
}

async fn parse_data(driver: &WebDriver) -> anyhow::Result<Vec<Company>> {
    let source = driver.source().await?;
    let document = scraper::Html::parse_document(&source);
    let symbol_selector = scraper::Selector::parse(SYMBOL_SELECTOR)
        .map_err(|e| eprintln!("{e}"))
        .unwrap();
    let names_selector = scraper::Selector::parse(COMPANY_NAME_SELECTOR)
        .map_err(|e| eprintln!("{e}"))
        .unwrap();
    let symbols: Vec<String> = document
        .select(&symbol_selector)
        .map(|e| e.inner_html())
        .collect();
    let names: Vec<String> = document
        .select(&names_selector)
        .map(|e| e.inner_html())
        .collect();
    assert_eq!(symbols.len(), names.len());

    let companies: Vec<Company> = symbols.iter().zip(names.iter()).map(|(s, n)| {
        Company { symbol: s.clone(), name: n.clone() }
    }).collect();

    Ok(companies)
}

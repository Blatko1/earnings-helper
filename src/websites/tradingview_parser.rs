use std::{time::Duration, vec};
use thirtyfour::{prelude::ElementQueryable, By, WebDriver};

use super::{Company, TRADINGVIEW};
use crate::RelativeDay;

const DAY_SELECTOR: &str = "div[class=\"itemContent-LeZwGiB6\"]";
const SYMBOL_SELECTOR: &str =
    "a[class=\"tv-screener__symbol apply-common-tooltip\"]";
const COMPANY_NAME_SELECTOR: &str = "span[class=\"tv-screener__description\"]";
const WAIT_INTERVAL: Duration = Duration::from_secs(1);
const LOAD_WAIT: Duration = Duration::from_secs(2);
const TIMEOUT_FIVE_SEC: Duration = Duration::from_secs(5);
const SCROLL_INTO_VIEW: &str =
    r#"arguments[0].scrollIntoView({behavior: "auto", block: "center"});"#;

pub async fn get_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    driver.goto(TRADINGVIEW).await?;

    match day {
        RelativeDay::Yesterday => to_previous_day(driver).await?,
        RelativeDay::Tomorrow => to_next_day(driver).await?,
        _ => (),
    }
    // Wait for the browser to load data table
    tokio::time::sleep(LOAD_WAIT).await;

    parse_data(driver).await
}

async fn to_previous_day(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(DAY_SELECTOR))
        .with_text("Yesterday")
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

async fn to_next_day(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(DAY_SELECTOR))
        .with_text("Tomorrow")
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
    let names_html: Vec<String> = document
        .select(&names_selector)
        .map(|e| e.inner_html())
        .collect();
    let names: Vec<String> = names_html
        .iter()
        .map(|n| {
            n.split('<')
                .collect::<Vec<&str>>()
                .first()
                .unwrap()
                .trim()
                .to_string()
        })
        .collect();
    assert_eq!(symbols.len(), names_html.len());

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

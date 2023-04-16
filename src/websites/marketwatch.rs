use chrono::Datelike;
use chrono::NaiveDate;
use chrono::Weekday;
use thirtyfour::prelude::ElementWaitable;
use thirtyfour::By;
use thirtyfour::WebDriver;

use crate::RelativeDay;

use super::Company;
use super::MARKETWATCH;

pub async fn get_marketwatch_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    driver.goto(MARKETWATCH).await?;
    accept_cookies(driver).await?;

    let today = chrono::offset::Local::now().date_naive();
    let target = day.get_date();
    let target_weekday = target.weekday();
    match today.weekday() {
        // Check if the target week is before the today week
        Weekday::Mon => {
            if target_weekday == Weekday::Sun {
                to_previous_week(driver).await?;
            }
        }
        // Check if the target week is before the today week
        Weekday::Sun => {
            if target_weekday == Weekday::Mon {
                to_next_week(driver).await?;
            }
        }
        _ => (),
    }
    get_data_for_date(driver, target).await
}

async fn to_previous_week(driver: &WebDriver) -> anyhow::Result<()> {
    // Firstly, click the previous day button if available
    if let Err(_) = driver
        .find(By::Css(PREVIOUS_DAY_SELECTOR))
        .await?
        .click()
        .await
    {
        // If it was not available, click the previous day button.
        driver
            .find(By::Css(PREVIOUS_WEEK_SELECTOR))
            .await?
            .click()
            .await?;
    }
    Ok(())
}

async fn to_next_week(driver: &WebDriver) -> anyhow::Result<()> {
    // Firstly, click the next day button if available
    if let Err(_) = driver.find(By::Css(NEXT_DAY_SELECTOR)).await?.click().await
    {
        // If it was not available, click the next day button.
        driver
            .find(By::Css(NEXT_WEEK_SELECTOR))
            .await?
            .click()
            .await?;
    }
    Ok(())
}

/// Waits for and accepts cookies in order to be able to
/// interact with the elements behind the cookies iframe.
async fn accept_cookies(driver: &WebDriver) -> anyhow::Result<()> {
    let body = driver.find(By::Css(BODY_SELECTOR)).await?;
    body.wait_until().displayed().await?;
    let cookies_iframe = driver.find(By::Id(COOKIE_MESSAGE_CONTAINER)).await?;
    cookies_iframe.wait_until().displayed().await?;
    let iframe = driver.find(By::Id(COOKIE_MESSAGE_IFRAME)).await?;
    iframe.enter_frame().await?;
    driver
        .find(By::Css(COOKIES_AGREE_BUTTON_SELECTOR))
        .await?
        .click()
        .await?;
    driver.enter_default_frame().await?;

    Ok(())
}

async fn get_data_for_date(
    driver: &WebDriver,
    date: NaiveDate,
) -> anyhow::Result<Vec<Company>> {
    let source = driver.source().await?;
    let document = scraper::Html::parse_document(&source);
    let date_selector = &format!(
        "div.element[data-tab-pane=\"{}\"]>",
        date.format("%m/%d/%Y")
    );

    let css_symbol_selector = &format!("{}{}", date_selector, SYMBOL_SELECTOR);
    let css_company_name_selector =
        &format!("{}{}", date_selector, COMPANY_NAME_SELECTOR);
    let symbol_selector = scraper::Selector::parse(css_symbol_selector)
        .map_err(|e| eprintln!("{e}"))
        .unwrap();
    let company_name_selector =
        scraper::Selector::parse(css_company_name_selector)
            .map_err(|e| eprintln!("{e}"))
            .unwrap();

    let symbols: Vec<String> = document
        .select(&symbol_selector)
        .map(|e| e.inner_html())
        .collect();
    let company_names: Vec<String> = document
        .select(&company_name_selector)
        .map(|e| e.inner_html())
        .collect();
    assert_eq!(symbols.len(), company_names.len());
    let companies: Vec<Company> = symbols
        .iter()
        .zip(company_names.iter())
        .map(|(symbol, name)| Company {
            symbol: symbol.clone(),
            name: name.clone(),
        })
        .collect();

    Ok(companies)
}

// Children of the parent div with date.
const SYMBOL_SELECTOR: &str =
    "div>div>table>tbody>tr>td[class=\"overflow__cell align--left\"]>div>a";
const COMPANY_NAME_SELECTOR: &str =
    "div>div>table>tbody>tr>td[class=\"overflow__cell fixed--column align--left\"]>div[class=\"cell__content fixed--cell\"]>a";
const PREVIOUS_WEEK_SELECTOR: &str = "li[class=\"tab__item prev week\"]";
const NEXT_WEEK_SELECTOR: &str = "li[class=\"tab__item next week\"]";
const PREVIOUS_DAY_SELECTOR: &str = "li[class=\"tab__item prev day\"]";
const NEXT_DAY_SELECTOR: &str = "li[class=\"tab__item next day\"]";
const COOKIES_AGREE_BUTTON_SELECTOR: &str = "button[class=\"message-component message-button no-children focusable agree-btn sp_choice_type_11\"]";
const COOKIE_MESSAGE_CONTAINER: &str = "sp_message_container_719544";
const COOKIE_MESSAGE_IFRAME: &str = "sp_message_iframe_719544";
const BODY_SELECTOR: &str = "body[class=\"page--tools   \"]";

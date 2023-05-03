use chrono::{Datelike, NaiveDate, Weekday};
use thirtyfour::{prelude::ElementQueryable, By, WebDriver};

use super::{Company, MARKETWATCH, TIMEOUT_FIVE_SEC, WAIT_INTERVAL};
use crate::{websites::TIMEOUT_TEN_SEC, RelativeDay};

const SYMBOL_SELECTOR: &str =
    "div>div>table>tbody>tr>td[class=\"overflow__cell align--left\"]>div>a";
const COMPANY_NAME_SELECTOR: &str =
    "div>div>table>tbody>tr>td[class=\"overflow__cell fixed--column align--left\"]>div[class=\"cell__content fixed--cell\"]>a";
const PREVIOUS_WEEK_SELECTOR: &str = "li[class=\"tab__item prev week\"]";
const NEXT_WEEK_SELECTOR: &str = "li[class=\"tab__item next week\"]";
const PREVIOUS_DAY_SELECTOR: &str = "li[class=\"tab__item prev day\"]";
const NEXT_DAY_SELECTOR: &str = "li[class=\"tab__item next day\"]";
const COOKIES_AGREE_BUTTON_SELECTOR: &str = "button[class=\"message-component message-button no-children focusable agree-btn sp_choice_type_11\"]";
const COOKIE_MESSAGE_IFRAME: &str = "sp_message_iframe_719544";

pub async fn get_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    driver.goto(MARKETWATCH).await?;
    // If cookies window was not found then make sure to return
    // back to the default frame.
    accept_cookies(driver)
        .await
        .or(driver.enter_default_frame().await)?;

    let today = chrono::offset::Local::now().date_naive();
    let target = day.get_date();
    let target_weekday = target.weekday();
    // Weekdays here are counted from Mon to Sun.
    match today.weekday() {
        // Check if the target week is before the today week
        Weekday::Mon => {
            if target_weekday == Weekday::Sun {
                to_previous_week(driver).await?;
            }
        }
        // Check if the target week is after the today week
        Weekday::Sun => {
            if target_weekday == Weekday::Mon {
                to_next_week(driver).await?;
            }
        }
        _ => (),
    }
    parse_data(driver, target).await
}

async fn to_previous_week(driver: &WebDriver) -> anyhow::Result<()> {
    // Firstly, click the previous day button if available
    if driver
        .find(By::Css(PREVIOUS_DAY_SELECTOR))
        .await?
        .click()
        .await
        .is_err()
    {
        // If it was not available, click the previous week button.
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
    if driver
        .find(By::Css(NEXT_DAY_SELECTOR))
        .await?
        .click()
        .await
        .is_err()
    {
        // If it was not available, click the next week button.
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
    let iframe = driver
        .query(By::Id(COOKIE_MESSAGE_IFRAME))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Wait for cookies dialog box to appear")
        .single()
        .await?;
    iframe.enter_frame().await?;
    driver
        .find(By::Css(COOKIES_AGREE_BUTTON_SELECTOR))
        .await?
        .click()
        .await?;
    driver.enter_default_frame().await?;

    Ok(())
}

async fn parse_data(
    driver: &WebDriver,
    date: NaiveDate,
) -> anyhow::Result<Vec<Company>> {
    let date_selector =
        &format!("div.element[data-tab-pane=\"{}\"]", date.format("%m/%d/%Y"));
    driver
        .query(By::Css(date_selector))
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .desc("Find the current date data")
        .single()
        .await?;
    let source = driver.source().await?;
    let document = scraper::Html::parse_document(&source);

    let css_symbol_selector = &format!("{}>{}", date_selector, SYMBOL_SELECTOR);
    let css_company_name_selector =
        &format!("{}>{}", date_selector, COMPANY_NAME_SELECTOR);
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

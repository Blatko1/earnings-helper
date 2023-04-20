use chrono::{Datelike, Days, NaiveDate, Weekday};
use std::{time::Duration, vec};
use thirtyfour::{prelude::ElementQueryable, By, WebDriver};

use super::{Company, ZACKS};
use crate::RelativeDay;

const PREVIOUS_WEEK_SELECTOR: &str = "div[class=\"prenext_txt align_left\"]>a";
const NEXT_WEEK_SELECTOR: &str = "div[class=\"prenext_txt align_right\"]>a";
const WAIT_INTERVAL: Duration = Duration::from_secs(1);
const TIMEOUT_FIVE_SEC: Duration = Duration::from_secs(5);
const TIMEOUT_TEN_SEC: Duration = Duration::from_secs(10);
const SCROLL_INTO_VIEW: &str =
    r#"arguments[0].scrollIntoView({behavior: "auto", block: "center"});"#;
const COOKIE_ACCEPT_ID: &str = "accept_cookie";
const EVENTS_TITLE: &str = "WeeklyEventsTitle";
const COPY_BUTTON_SELECTOR: &str =
    "a[class=\"dt-button buttons-copy buttons-html5\"]";

pub async fn get_zacks_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    driver.goto(ZACKS).await?;
    // Accept cookies in order to remove the cookies 'obstacle' dialog box.
    accept_cookies(driver).await?;

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

    get_data(driver, target).await
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
    driver
        .query(By::Id(COOKIE_ACCEPT_ID))
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .desc("Find the 'Accept cookies' button")
        .single()
        .await?
        .click()
        .await?;
    Ok(())
}

async fn get_data(
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
    let button = driver
        .query(By::Css(COPY_BUTTON_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .single()
        .await?;
    driver
    .execute(SCROLL_INTO_VIEW, vec![button.to_json()?])
    .await?;
    button.click().await?;
    let mut clipboard = arboard::Clipboard::new()?;
    let data = clipboard.get_text()?;

    Ok(vec![])
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

mod websites;

use crate::websites::RelativeDay;
use std::io::Write;
use thirtyfour::{DesiredCapabilities, WebDriver};
use websites::Company;

const MINIMUM_REFERENCES: usize = 5;

#[tokio::main]
async fn main() {
    print!("Initializing WebDriver...");
    std::io::stdout().flush().unwrap();
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    let driver = WebDriver::new("http://localhost:9515", caps)
        .await
        .map_err(|e| println!("Is chromedriver started? Error: {e}"))
        .unwrap();
    println!("Success!");

    let day = RelativeDay::Tomorrow;

    let marketwatch_data =
        websites::marketwatch_data(&driver, day).await.unwrap();
    let zacks_data = websites::zacks_data(&driver, day).await.unwrap();
    let tradingview_data =
        websites::tradingview_data(&driver, day).await.unwrap();
    let investing_data = websites::investing_data(&driver, day).await.unwrap();
    let benzinga_data = websites::benzinga_data(&driver, day).await.unwrap();

    driver.quit().await.unwrap();

    print!("Evaluating parsed companies...");
    let candidates = eval_candidates(vec![
        marketwatch_data,
        zacks_data,
        tradingview_data,
        investing_data,
        benzinga_data,
    ]);

    println!("FINALI: {candidates:?}")
}

/// Evaluate candidates by data corelation.
fn eval_candidates(data: Vec<Vec<Company>>) -> Vec<CompanyCandidate> {
    let capacity = data.iter().map(|d| d.len()).sum();
    let avg = capacity / data.len();

    // Holds all parsed entries do the name is 'duplicate candidates'.
    let mut dup_candidates = Vec::with_capacity(capacity);
    for d in data.into_iter() {
        for c in d.into_iter() {
            dup_candidates.push(c)
        }
    }

    let mut result = Vec::with_capacity(avg);
    loop {
        let mut references: usize = 1;
        let company = dup_candidates.swap_remove(0);
        loop {
            if let Some(i) = dup_candidates.iter().position(|c| c.eq(&company)) {
                references = references + 1;
                dup_candidates.swap_remove(i);
                continue;
            }
            break;
        }
        if references >= MINIMUM_REFERENCES {
            result.push(CompanyCandidate {
                company,
                refs: references,
            })
        }
        if dup_candidates.is_empty() {
            break;
        }
    }
    println!(" Success!");
    println!("Number of parsed entries: {capacity}");

    result
}

#[derive(Debug, Ord, PartialEq, Eq, PartialOrd)]
struct CompanyCandidate {
    company: Company,
    refs: usize,
}

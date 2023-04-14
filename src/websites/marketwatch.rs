use chrono::{DateTime, Local};

use super::Company;
use super::MARKETWATCH;

pub fn get_marketwatch_data(date: DateTime<Local>) -> anyhow::Result<Vec<Company>> {
    let req = reqwest::blocking::get(MARKETWATCH)?;
    let document = scraper::Html::parse_document(&req.text()?);
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

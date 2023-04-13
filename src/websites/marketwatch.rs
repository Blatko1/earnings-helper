use chrono::NaiveDate;

use super::MARKETWATCH;

pub fn get_marketwatch_data(date: NaiveDate) -> anyhow::Result<()> {
    let req = reqwest::blocking::get(MARKETWATCH)?;
    let document = scraper::Html::parse_document(&req.text()?);
    let main_parent = &format!(
        "div.element[data-tab-pane=\"{}\"]>",
        date.format("%m/%d/%Y")
    );

    let css_symbol_selector = &format!("{}{}", main_parent, SYMBOL_SELECTOR);
    let css_company_name_selector =
        &format!("{}{}", main_parent, COMPANY_NAME_SELECTOR);
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
    println!("{symbols:?} \n {company_names:?}");

    Ok(())
}

// Children of the parent div with date.
const SYMBOL_SELECTOR: &str =
    "div>div>table>tbody>tr>td[class=\"overflow__cell align--left\"]>div>a";
const COMPANY_NAME_SELECTOR: &str =
    "div>div>table>tbody>tr>td[class=\"overflow__cell fixed--column align--left\"]>div[class=\"cell__content fixed--cell\"]>a";

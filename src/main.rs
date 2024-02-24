mod commands;
mod parser;

use crate::parser::RelativeDay;
use chrono::{DateTime, Days, Local};
use parser::Company;
use std::io::Write;

const OUTPUT_FILE_NAME: &str = "company_candidates.txt";

#[derive(Debug, Ord, PartialEq, Eq, PartialOrd)]
struct CompanyCandidate {
    company: Company,
    refs: usize,
}

#[tokio::main]
async fn main() {
    let mut stdout = std::io::stdout().lock();
    let matches = commands::cmd().get_matches();

    write!(stdout, "Data parsing in progress for: ").unwrap();

    let date = Local::now();
    let day = if matches.get_flag("tmr") {
        let tomorrow_date = date.checked_add_days(Days::new(1)).unwrap().date_naive();
        write!(stdout, "TOMORROW ({})", tomorrow_date).unwrap();
        RelativeDay::Tomorrow
    } else if matches.get_flag("yda") {
        let yesterday_date = date.checked_sub_days(Days::new(1)).unwrap().date_naive();
        write!(stdout, "YESTERDAY ({})", yesterday_date).unwrap();
        RelativeDay::Yesterday
    } else {
        write!(stdout, "TODAY ({})", date.date_naive()).unwrap();
        RelativeDay::Today
    };
    let window_visibility = matches.get_flag("preview");

    let min_references = *matches.get_one::<u8>("refs").unwrap() as usize;
    write!(stdout, "\nMinimum references: {min_references}\n").unwrap();

    let (data, parsed_websites) =
        parser::parse_website_data(day, window_visibility)
            .await
            .unwrap();

    write!(
        stdout,
        "\nSuccessfully parsed websites: {} out of 5",
        parsed_websites
    )
    .unwrap();
    if data.is_empty() {
        write!(
            stdout,
            "\nTotal number of entries: 0",
        )
        .unwrap();
        std::process::exit(0);
    };
    let avg = parsed_websites / data.len();
    write!(
        stdout,
        "\nTotal number of entries (no filter): {}",
        data.len()
    )
    .unwrap();

    write!(
        stdout,
        "\nEntries with less than {} references will be filtered.",
        min_references
    )
    .unwrap();
    write!(stdout, "\nEvaluating parsed companies...").unwrap();
    stdout.flush().unwrap();
    let mut candidates = eval_candidates(data, min_references, avg);
    candidates.sort_by(|a, b| a.refs.cmp(&b.refs).reverse());
    write!(stdout, " Done!").unwrap();
    write!(
        stdout,
        "\nNumber of entries after filtering: {}\n",
        candidates.len()
    )
    .unwrap();
    std::io::stdout().flush().unwrap();

    data_file_output(candidates).unwrap();
}

/// Evaluate candidates by data corelation. If the parsed company has
/// multiple duplicates (references) it will be shown if it's more than
/// the MINIMUM_REFERENCES.
///
/// `avg` - represents average number of companies parsed per website,
/// needed for allocating space for [`Vec::with_capacity()`]
fn eval_candidates(
    mut data: Vec<Company>,
    min_refs: usize,
    avg: usize,
) -> Vec<CompanyCandidate> {
    let mut result = Vec::with_capacity(avg);
    loop {
        let mut references: usize = 1;
        let mut company = data.swap_remove(0);
        loop {
            if let Some(i) = data.iter().position(|c| c.eq(&company)) {
                references += 1;
                let dup = data.swap_remove(i);
                // If variable's 'company' name is empty insert duplicate's name.
                if company.name.is_empty() {
                    company.name = dup.name;
                }
                continue;
            }
            break;
        }
        if references >= min_refs {
            result.push(CompanyCandidate {
                company,
                refs: references,
            })
        }
        if data.is_empty() {
            break;
        }
    }
    result
}

fn data_file_output(data: Vec<CompanyCandidate>) -> anyhow::Result<()> {
    let mut output = String::new();
    output.push_str("Refs.\tSymbol \tCompany Name\n");
    for d in data.into_iter() {
        output.push_str(&format!(
            "{:>5}\t{:<7}\t{}\n",
            d.refs, d.company.symbol, d.company.name
        ));
    }
    std::fs::write(OUTPUT_FILE_NAME, output).unwrap();
    println!(
        "Parsed data saved at:\n\t'{}'",
        std::fs::canonicalize(OUTPUT_FILE_NAME)?.display()
    );
    Ok(())
}

mod parser;

use crate::parser::RelativeDay;
use parser::Company;
use std::io::Write;

const MINIMUM_REFERENCES: usize = 1;

#[derive(Debug, Ord, PartialEq, Eq, PartialOrd)]
struct CompanyCandidate {
    company: Company,
    refs: usize,
}

#[tokio::main]
async fn main() {
    let (data, avg) = parser::parse_website_data().await.unwrap();

    println!("Number of entries (no filter): {}", data.len());
    println!(
        "Filtering entries with minimum of {} references.",
        MINIMUM_REFERENCES
    );

    print!("Evaluating parsed companies...");
    let candidates = eval_candidates(data, avg);
    println!(" Done!");
    println!("Number of entries after filtering: {}", candidates.len());
    std::io::stdout().flush().unwrap();

    println!("FINALI: {candidates:?}")
}

/// Evaluate candidates by data corelation.
///
/// `avg` - represents average number of companies parsed per website,
/// needed for allocating space for [`Vec::with_capacity()`]
fn eval_candidates(
    mut data: Vec<Company>,
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
        if references >= MINIMUM_REFERENCES {
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

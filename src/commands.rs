use clap::{Arg, ArgAction, Command};

pub fn cmd() -> Command {
    Command::new("Company Earnings Calendar Parser")
        .version("1.0")
        .author("Blatko1")
        .about(
            "Parses data about upcoming company \
            earnings from 5 different websites.",
        )
        .arg(
            Arg::new("refs")
                .help(
                    "Set the minimum amount of references \
                    needed for each company.",
                )
                .required(true)
                .value_parser(clap::value_parser!(u8).range(..=5)),
        )
        .arg(
            Arg::new("preview")
                .short('p')
                .alias("view")
                .help(
                    "If present, the headless chrome will be visible \
                    to user while it's navigating and parsing data.",
                )
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("tdy")
                .short('n')
                .long("today")
                .alias("now")
                .help(
                    "Exclusive flag. Sets the parser to parse \
                    the today \nscheduled earnings \
                    calendar data from websites.",
                )
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["tmr", "yda"]),
        )
        .arg(
            Arg::new("tmr")
                .short('t')
                .long("tomorrow")
                .help(
                    "Exclusive flag. Sets the parser to parse \
                    the tomorrow \nscheduled earnings \
                    calendar data from websites.",
                )
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["tdy", "yda"]),
        )
        .arg(
            Arg::new("yda")
                .short('y')
                .long("yesterday")
                .help(
                    "Exclusive flag. Sets the parser to parse \
                    the yesterday \nscheduled earnings \
                    calendar data from websites.",
                )
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["tdy", "tmr"]),
        )
}

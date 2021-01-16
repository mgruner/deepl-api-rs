//! This binary crate provides a unix-style commandline application for integrating the
//! [DeepL API](https://www.deepl.com/docs-api/) into toolchains without any programming effort.
//!
//! *If you are looking for the `deepl-api` library crate, please refer
//! to [its documentation](../deepl_api/index.html) instead.*
//!
//! # Requirements
//!
//! You need to have a valid [DeepL Pro Developer](https://www.deepl.com/pro#developer) account
//! with an associated API key. This key must be made available to the application, e. g. via
//! environment variable:
//!
//! ```bash
//! export DEEPL_API_KEY=YOUR_KEY
//! ```
//!
//! # Examples
//!
//! ## Overview
//!
//! To get an overview of the available commands, just invoke the program.
//!
//! ```text
//! shell> deepl
//! deepl 0.1.0
//! Command line client for the DeepL API
//!
//! USAGE:
//!     deepl <SUBCOMMAND>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!     help                 Prints this message or the help of the given subcommand(s)
//!     languages            Fetch list of available source and target languages
//!     translate            A subcommand for controlling testing
//!     usage-information    Fetch imformation about account usage & limits
//! ```
//!
//! You can call `deepl help translate` to get a detailed reference for the various options of the
//! `translate` command, for example.
//!
//! ## Translating Text
//!
//! By default, `deepl` reads from `STDIN` and writes to `STDOUT`, which means that you can integrate
//! it nicely into toolchains.
//!
//! ```text
//! shell> echo "Please go home." | deepl translate --source-language EN --target-language DE | cat -
//! Bitte gehen Sie nach Hause.
//! ```
//!
//! By providing the options `--input-file` and / or `--output-file`, you can tell `deepl` to
//! read from / write to files, rather than `STDIN` / `STDOUT`.
//!
//! ## Retrieving Account Usage & Limits
//!
//! ```text
//! shell> deepl usage-information
//! Available characters per billing period: 250000
//! Characters already translated in the current billing period: 3317
//! ```
//!
//! ## Retrieving Available Languages
//!
//! ```text
//! shell> deepl languages
//! DeepL can translate from the following source languages:
//!   DE    (German)
//!   EN    (English)
//!   ES    (Spanish)
//!   ...
//!
//! DeepL can translate to the following target languages:
//!   DE    (German)
//!   EN-GB (English (British))
//!   EN-US (English (American))
//!   ES    (Spanish)
//!   ...
//! ```

use deepl_api::*;
use std::fs;
use std::io::{self, Read};

mod parse_arguments;
use parse_arguments::*;

fn main() {
    let opts: Opts = Opts::parse();

    let key = match std::env::var("DEEPL_API_KEY") {
        Ok(val) if val.len() > 0 => val,
        _ => {
            eprintln!("Error: no DEEPL_API_KEY found. Please provide your API key in this environment variable.");
            std::process::exit(1);
        }
    };

    let deepl = DeepL::new(key);

    let result = match opts.subcmd {
        SubCommand::Translate(t) => translate(&deepl, &t),
        SubCommand::UsageInformation => usage_information(&deepl),
        SubCommand::Languages => languages(&deepl),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1)
    }
}

fn translate(deepl: &DeepL, t: &Translate) -> Result<()> {
    let mut t_opts = TranslationOptions {
        split_sentences: None,
        preserve_formatting: None,
        formality: None,
    };
    if t.preserve_formatting {
        t_opts.preserve_formatting = Some(true);
    }
    if t.formality_less {
        t_opts.formality = Some(Formality::Less);
    }
    if t.formality_more {
        t_opts.formality = Some(Formality::More);
    }

    let mut text = String::new();
    if let Some(filepath) = t.input_file.clone() {
        text = fs::read_to_string(filepath)?;
    } else {
        io::stdin().read_to_string(&mut text)?;
    }

    let texts = TranslatableTextList {
        source_language: t.source_language.clone(),
        target_language: t.target_language.clone(),
        texts: vec![text],
    };

    let translations = deepl.translate(Some(t_opts), texts)?;
    let mut output = String::new();
    for t in translations {
        output.push_str(&t.text);
    }

    if let Some(filepath) = t.output_file.clone() {
        fs::write(filepath, &output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}

fn usage_information(deepl: &DeepL) -> Result<()> {
    let usage = deepl.usage_information()?;
    println!(
        "Available characters per billing period: {}",
        usage.character_limit
    );
    println!(
        "Characters already translated in the current billing period: {}",
        usage.character_count
    );
    Ok(())
}

fn languages(deepl: &DeepL) -> Result<()> {
    let source_langs = deepl.source_languages()?;
    let target_langs = deepl.target_languages()?;
    println!("DeepL can translate from the following source languages:");
    for lang in source_langs {
        println!("  {:<5} ({})", lang.language, lang.name)
    }
    println!();
    println!("DeepL can translate to the following target languages:");
    for lang in target_langs {
        println!("  {:<5} ({})", lang.language, lang.name)
    }
    Ok(())
}

//! Provides a lightweight wrapper for the DeepL Pro REST API.
//!
//! *If you are looking for the `deepl` commandline utility, please refer
//! to [its documentation](../deepl/index.html) instead.*
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
//! # Example
//!
//! ```rust
//! use deepl_api::*;
//!
//! // Create a DeepL instance for our account.
//! let deepl = DeepL::new(std::env::var("DEEPL_API_KEY").unwrap());
//!
//! // Translate Text
//! let texts = TranslatableTextList {
//!     source_language: Some("DE".to_string()),
//!     target_language: "EN-US".to_string(),
//!     texts: vec!("ja".to_string()),
//! };
//! let translated = deepl.translate(None, texts).unwrap();
//! assert_eq!(translated[0].text, "yes");
//!
//! // Fetch Usage Information
//! let usage_information = deepl.usage_information().unwrap();
//! assert!(usage_information.character_limit > 0);
//! ```
//!
//! # See Also
//!
//! The main API functions are documented in the [DeepL] struct.

use chrono::{DateTime, Utc};
use error_chain::*;
use reqwest::{self, Method, blocking::Response};
use serde::Deserialize;

/// Information about API usage & limits for this account.
#[derive(Debug, Deserialize)]
pub struct UsageInformation {
    /// How many characters can be translated per billing period, based on the account settings.
    pub character_limit: u64,
    /// How many characters were already translated in the current billing period.
    pub character_count: u64,
}

/// Information about available languages.
pub type LanguageList = Vec<LanguageInformation>;

/// Information about a single language.
#[derive(Debug, Deserialize)]
pub struct LanguageInformation {
    /// Custom language identifier used by DeepL, e. g. "EN-US". Use this
    /// when specifying source or target language.
    pub language: String,
    /// English name of the language, e. g. `English (America)`.
    pub name: String,
}

/// Translation option that controls the splitting of sentences before the translation.
pub enum SplitSentences {
    /// Don't split sentences.
    None,
    /// Split on punctuation only.
    Punctuation,
    /// Split on punctuation and newlines.
    PunctuationAndNewlines,
}

/// Translation option that controls the desired translation formality.
pub enum Formality {
    /// Default formality.
    Default,
    /// Translate less formally.
    More,
    /// Translate more formally.
    Less,
}

/// Custom [flags for the translation request](https://www.deepl.com/docs-api/translating-text/request/).
pub struct TranslationOptions {
    /// Sets whether the translation engine should first split the input into sentences. This is enabled by default.
    pub split_sentences: Option<SplitSentences>,
    /// Sets whether the translation engine should respect the original formatting, even if it would usually correct some aspects.
    pub preserve_formatting: Option<bool>,
    /// Sets whether the translated text should lean towards formal or informal language.
    pub formality: Option<Formality>,
    /// Specify the glossary to use for the translation.
    pub glossary_id: Option<String>,
}

/// Format of glossary entries when creating a glossary.
pub enum GlossaryEntriesFormat {
    /// tab-separated values
    Tsv,
    /// comma-separated values
    Csv,
}

/// Representation of a glossary.
#[derive(Debug, Deserialize)]
pub struct Glossary {
    /// A unique ID assigned to a glossary.
    pub glossary_id: String,
    /// Name associated with the glossary.
    pub name: String,
    /// Indicates if the newly created glossary can already be used in translate requests. If the created glossary is not yet ready, you have to wait and check the ready status of the glossary before using it in a translate request.
    pub ready: bool,
    /// The language in which the source texts in the glossary are specified.
    pub source_lang: String,
    /// The language in which the target texts in the glossary are specified.
    pub target_lang: String,
    /// The creation time of the glossary.
    pub creation_time: DateTime<Utc>,
    /// The number of entries in the glossary.
    pub entry_count: u64,
}
// Representation of a glossary listing response.
#[derive(Debug, Deserialize)]
pub struct GlossaryListing {
    /// A list of glossaries.
    pub glossaries: Vec<Glossary>,
}

/// Holds a list of strings to be translated.
#[derive(Debug, Deserialize)]
pub struct TranslatableTextList {
    /// Source language, if known. Will be auto-detected by the DeepL API
    /// if not provided.
    pub source_language: Option<String>,
    /// Target language (required).
    pub target_language: String,
    /// List of texts that are supposed to be translated.
    pub texts: Vec<String>,
}

/// Holds one unit of translated text.
#[derive(Debug, Deserialize, PartialEq)]
pub struct TranslatedText {
    /// Source language. Holds the value provided, or otherwise the value that DeepL auto-detected.
    pub detected_source_language: String,
    /// Translated text.
    pub text: String,
}

// Only needed for JSON deserialization.
#[derive(Debug, Deserialize)]
struct TranslatedTextList {
    translations: Vec<TranslatedText>,
}

// Only needed for JSON deserialization.
#[derive(Debug, Deserialize)]
struct ServerErrorMessage {
    message: String,
    detail: Option<String>
}

/// The main API entry point representing a DeepL developer account with an associated API key.
///
/// # Example
///
/// See [Example](crate#example).
///
/// # Error Handling
///
/// None of the functions will panic. Instead, the API methods usually return a [Result<T>] which may
/// contain an [Error] of one of the defined [ErrorKinds](ErrorKind) with more information about what went wrong.
///
/// If you get an [AuthorizationError](ErrorKind::AuthorizationError), then something was wrong with your API key, for example.
pub struct DeepL {
    api_key: String,
}

/// Implements the actual REST API. See also the [online documentation](https://www.deepl.com/docs-api/).
impl DeepL {
    /// Use this to create a new DeepL API client instance where multiple function calls can be performed.
    /// A valid `api_key` is required.
    ///
    /// Should you ever need to use more than one DeepL account in our program, then you can create one
    /// instance for each account / API key.
    pub fn new(api_key: String) -> DeepL {
        DeepL { api_key }
    }

    /// Private method that performs the HTTP calls.
    fn http_request(
        &self,
        method: Method,
        url: &str,
        params: Option<&[(&str, std::string::String)]>,
    ) -> Result<reqwest::blocking::Response> {

        let url = match self.api_key.ends_with(":fx") {
            true  => format!("https://api-free.deepl.com/v2{}", url),
            false => format!("https://api.deepl.com/v2{}", url),
        };

        let client = reqwest::blocking::Client::new();
        let request = client.request(method.clone(), &url).header("Authorization", format!("DeepL-Auth-Key {}", self.api_key));

        let response = match params {
            Some(params) => {
                match method {
                    Method::GET => request.query(params).send(),
                    Method::PATCH | Method::POST | Method::PUT => {
                        request.form(params).send()
                    },
                    _ => unreachable!("Only GET, PATCH, POST and PUT are supported with params."),
                }
            },
            None => request.send(),
        };

        let res = match response {
            Ok(response) if response.status().is_success() => response,
            Ok(response) if response.status() == reqwest::StatusCode::UNAUTHORIZED => {
                bail!(ErrorKind::AuthorizationError)
            }
            Ok(response) if response.status() == reqwest::StatusCode::FORBIDDEN => {
                bail!(ErrorKind::AuthorizationError)
            }
            Ok(response) if response.status() == reqwest::StatusCode::NOT_FOUND => {
                bail!(ErrorKind::NotFoundError)
            }
            // DeepL sends back error messages in the response body.
            //   Try to fetch them to construct more helpful exceptions.
            Ok(response) => {
                let status = response.status();
                match response.json::<ServerErrorMessage>() {
                    Ok(server_error) => bail!(ErrorKind::ServerError(format!("{}: {}", server_error.message, server_error.detail.unwrap_or_default()))),
                    _ => bail!(ErrorKind::ServerError(status.to_string())),
                }
            }
            Err(e) => {
                bail!(e)
            }
        };
        Ok(res)
    }

    /// Retrieve information about API usage & limits.
    /// This can also be used to verify an API key without consuming translation contingent.
    ///
    /// See also the [vendor documentation](https://www.deepl.com/docs-api/other-functions/monitoring-usage/).
    pub fn usage_information(&self) -> Result<UsageInformation> {
        let res = self.http_request(Method::POST, "/usage", None)?;

        match res.json::<UsageInformation>() {
            Ok(content) => return Ok(content),
            _ => {
                bail!(ErrorKind::DeserializationError);
            }
        };
    }

    /// Retrieve all currently available source languages.
    ///
    /// See also the [vendor documentation](https://www.deepl.com/docs-api/other-functions/listing-supported-languages/).
    pub fn source_languages(&self) -> Result<LanguageList> {
        return self.languages("source");
    }

    /// Retrieve all currently available target languages.
    ///
    /// See also the [vendor documentation](https://www.deepl.com/docs-api/other-functions/listing-supported-languages/).
    pub fn target_languages(&self) -> Result<LanguageList> {
        return self.languages("target");
    }

    /// Private method to make the API calls for the language lists.
    fn languages(&self, language_type: &str) -> Result<LanguageList> {
            let res = self.http_request(Method::POST, "/languages", Some(&[("type", language_type.to_string())]))?;

        match res.json::<LanguageList>() {
            Ok(content) => return Ok(content),
            _ => bail!(ErrorKind::DeserializationError),
        }
    }

    /// Translate one or more [text chunks](TranslatableTextList) at once. You can pass in optional
    /// [translation flags](TranslationOptions) if you need non-default behaviour.
    ///
    /// Please see the parameter documentation and the
    /// [vendor documentation](https://www.deepl.com/docs-api/translating-text/) for details.
    pub fn translate(
        &self,
        options: Option<TranslationOptions>,
        text_list: TranslatableTextList,
    ) -> Result<Vec<TranslatedText>> {
        let mut query = vec![
            ("target_lang", text_list.target_language),
        ];
        if let Some(source_language_content) = text_list.source_language {
            query.push(("source_lang", source_language_content));
        }
        for text in text_list.texts {
            query.push(("text", text));
        }
        if let Some(opt) = options {
            if let Some(split_sentences) = opt.split_sentences {
                query.push((
                    "split_sentences",
                    match split_sentences {
                        SplitSentences::None => "0".to_string(),
                        SplitSentences::PunctuationAndNewlines => "1".to_string(),
                        SplitSentences::Punctuation => "nonewlines".to_string(),
                    },
                ));
            }
            if let Some(preserve_formatting) = opt.preserve_formatting {
                query.push((
                    "preserve_formatting",
                    match preserve_formatting {
                        false => "0".to_string(),
                        true => "1".to_string(),
                    },
                ));
            }
            if let Some(formality) = opt.formality {
                query.push((
                    "formality",
                    match formality {
                        Formality::Default => "default".to_string(),
                        Formality::More => "more".to_string(),
                        Formality::Less => "less".to_string(),
                    },
                ));
            }
            if let Some(glossary_id) = opt.glossary_id {
                query.push(("glossary_id", glossary_id));
            }
        }

        let res = self.http_request(Method::POST, "/translate", Some(&query))?;

        match res.json::<TranslatedTextList>() {
            Ok(content) => Ok(content.translations),
            _ => bail!(ErrorKind::DeserializationError),
        }
    }

    /// Create a glossary.
    ///
    /// Please take a look at the [vendor documentation](https://www.deepl.com/de/docs-api/glossaries/create-glossary/) for details.
    pub fn create_glossary(
        &self,
        name: String,
        source_lang: String,
        target_lang: String,
        entries: String,
        entries_format: GlossaryEntriesFormat
    ) -> Result<Glossary> {
        let res = self.http_request(Method::POST, "/glossaries", Some(&[
            ("name", name),
            ("source_lang", source_lang),
            ("target_lang", target_lang),
            ("entries", entries),
            ("entries_format", match entries_format {
                GlossaryEntriesFormat::Tsv => "tsv".to_string(),
                GlossaryEntriesFormat::Csv => "csv".to_string(),
            })])
        )?;

        match res.json::<Glossary>() {
            Ok(content) => Ok(content),
            _ => bail!(ErrorKind::DeserializationError),
        }
    }

    /// List all glossaries.
    ///
    /// Please take a look at the [vendor documentation](https://www.deepl.com/de/docs-api/glossaries/list-glossaries/) for details.
    pub fn list_glossaries(&self) -> Result<GlossaryListing> {
        let res = self.http_request(Method::GET, "/glossaries", None)?;

        match res.json::<GlossaryListing>() {
            Ok(content) => Ok(content),
            _ => bail!(ErrorKind::DeserializationError),
        }
    }

    /// Delete a glossary.
    ///
    /// Please take a look at the [vendor documentation](https://www.deepl.com/de/docs-api/glossaries/delete-glossary/) for details.
    pub fn delete_glossary(&self, glossary_id: String) -> Result<Response> {
        self.http_request(Method::DELETE, &format!("/glossaries/{}", glossary_id), None)
    }

    /// Retrieve Glossary Details.
    ///
    /// Please take a look at the [vendor documentation](https://www.deepl.com/de/docs-api/glossaries/get-glossary/) for details.
    pub fn get_glossary(&self, glossary_id: String) -> Result<Glossary> {
        let res = self.http_request(Method::GET, &format!("/glossaries/{}", glossary_id), None)?;

        match res.json::<Glossary>() {
            Ok(content) => Ok(content),
            _ => bail!(ErrorKind::DeserializationError),
        }
    }
}

mod errors {
    use error_chain::*;
    error_chain! {}
}

pub use errors::*;

error_chain! {
    foreign_links {
        IO(std::io::Error);
        Transport(reqwest::Error);
    }
    errors {
        /// Indicates that the provided API key was refused by the DeepL server.
        AuthorizationError {
            description("Authorization failed, is your API key correct?")
            display("Authorization failed, is your API key correct?")
        }
        /// An error occurred on the server side when processing a request. If possible, details
        /// will be provided in the error message.
        ServerError(message: String) {
            description("An error occurred while communicating with the DeepL server.")
            display("An error occurred while communicating with the DeepL server: '{}'.", message)
        }
        /// An error occurred on the client side when deserializing the response data.
        DeserializationError {
            description("An error occurred while deserializing the response data.")
            display("An error occurred while deserializing the response data.")
        }
        /// Resource was not found
        NotFoundError {
            description("The requested resource was not found.")
            display("The requested resource was not found.")
        }
    }

    skip_msg_variant
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_deepl() -> DeepL {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        DeepL::new(key)
    }

    #[test]
    fn usage_information() {
        let usage_information = create_deepl().usage_information().unwrap();
        assert!(usage_information.character_limit > 0);
    }

    #[test]
    fn source_languages() {
        let source_languages = create_deepl().source_languages().unwrap();
        assert_eq!(source_languages.last().unwrap().name, "Chinese");
    }

    #[test]
    fn target_languages() {
        let target_languages = create_deepl().target_languages().unwrap();
        assert_eq!(target_languages.last().unwrap().name, "Chinese (simplified)");
    }

    #[test]
    fn translate() {
        let deepl = create_deepl();
        let tests = vec![
            (
                None,
                TranslatableTextList {
                    source_language: Some("DE".to_string()),
                    target_language: "EN-US".to_string(),
                    texts: vec!["ja".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "DE".to_string(),
                    text: "yes".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: None,
                    preserve_formatting: Some(true),
                    glossary_id: None,
                    formality: None,
                }),
                TranslatableTextList {
                    source_language: Some("DE".to_string()),
                    target_language: "EN-US".to_string(),
                    texts: vec!["ja\n nein".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "DE".to_string(),
                    text: "yes\n no".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: Some(SplitSentences::None),
                    preserve_formatting: None,
                    glossary_id: None,
                    formality: None,
                }),
                TranslatableTextList {
                    source_language: Some("DE".to_string()),
                    target_language: "EN-US".to_string(),
                    texts: vec!["Ja. Nein.".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "DE".to_string(),
                    text: "Yes. No.".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: None,
                    preserve_formatting: None,
                    glossary_id: None,
                    formality: Some(Formality::More),
                }),
                TranslatableTextList {
                    source_language: Some("EN".to_string()),
                    target_language: "DE".to_string(),
                    texts: vec!["Please go home.".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "EN".to_string(),
                    text: "Bitte gehen Sie nach Hause.".to_string(),
                }],
            ),
            (
                Some(TranslationOptions {
                    split_sentences: None,
                    preserve_formatting: None,
                    glossary_id: None,
                    formality: Some(Formality::Less),
                }),
                TranslatableTextList {
                    source_language: Some("EN".to_string()),
                    target_language: "DE".to_string(),
                    texts: vec!["Please go home.".to_string()],
                },
                vec![TranslatedText {
                    detected_source_language: "EN".to_string(),
                    text: "Bitte geh nach Hause.".to_string(),
                }],
            ),
        ];
        for test in tests {
            assert_eq!(deepl.translate(test.0, test.1).unwrap(), test.2);
        }
    }

    #[test]
    #[should_panic(expected = "Error(ServerError(\"Parameter 'text' not specified.")]
    fn translate_empty() {
        let texts = TranslatableTextList {
            source_language: Some("DE".to_string()),
            target_language: "EN-US".to_string(),
            texts: vec![],
        };
        create_deepl().translate(None, texts).unwrap();
    }

    #[test]
    #[should_panic(expected = "Error(ServerError(\"Value for 'target_lang' not supported.")]
    fn translate_wrong_language() {
        let texts = TranslatableTextList {
            source_language: None,
            target_language: "NONEXISTING".to_string(),
            texts: vec!["ja".to_string()],
        };
        create_deepl().translate(None, texts).unwrap();
    }

    #[test]
    #[should_panic(expected = "Error(AuthorizationError")]
    fn translate_unauthorized() {
        let key = "wrong_key".to_string();
        let texts = TranslatableTextList {
            source_language: Some("DE".to_string()),
            target_language: "EN-US".to_string(),
            texts: vec!["ja".to_string()],
        };
        DeepL::new(key).translate(None, texts).unwrap();
    }

    #[test]
    fn glossaries() {
        let deepl = create_deepl();
        let glossary_name = "test_glossary".to_string();

        let mut glossary = deepl.create_glossary(
            glossary_name.clone(),
            "en".to_string(),
            "de".to_string(),
            "Action,Handlung".to_string(),
            GlossaryEntriesFormat::Csv
        ).unwrap();

        assert_eq!(glossary.name, glossary_name);
        assert_eq!(glossary.entry_count, 1);

        glossary = deepl.get_glossary(glossary.glossary_id).unwrap();
        assert_eq!(glossary.name, glossary_name);
        assert_eq!(glossary.entry_count, 1);

        let mut glossaries = deepl.list_glossaries().unwrap().glossaries;
        glossaries.retain(|glossary| glossary.name == glossary_name);
        let glossary = glossaries.pop().unwrap();
        assert_eq!(glossary.name, glossary_name);
        assert_eq!(glossary.entry_count, 1);

        assert_eq!(deepl.translate(
            Some(
                TranslationOptions {
                    split_sentences: None,
                    preserve_formatting: None,
                    glossary_id: Some(glossary.glossary_id.clone()),
                    formality: None,
                }
            ),
            TranslatableTextList {
                source_language: Some("en".to_string()),
                target_language: "de".to_string(),
                texts: vec!["Action".to_string()],
            }
        ).unwrap().pop().unwrap().text, "Handlung");

        deepl.delete_glossary(glossary.glossary_id.clone()).unwrap();
        let glossary_response = deepl.get_glossary(glossary.glossary_id);
        assert_eq!(glossary_response.unwrap_err().to_string(), crate::ErrorKind::NotFoundError.to_string());
    }
}

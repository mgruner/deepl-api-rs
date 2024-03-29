# deepl-api-rs

This repository contains a [Rust](https://www.rust-lang.org/) implementation of the [DeepL REST API](https://www.deepl.com/docs-api/).

## Contents

- A [Rust library crate](https://mgruner.github.io/deepl-api-rs-docs/deepl_api/index.html) for easy integration into Rust applications.
- The `deepl` [unix-style commandline application](https://mgruner.github.io/deepl-api-rs-docs/deepl/index.html) for integration into existing toolchains without any programming effort.
- Unit and integration tests.

Please refer to the linked documentation for instructions on how to get started with the API and/or the CLI tool.

## Features

- Query your account usage & limits information.
- Fetch the list of available source and target languages provided by DeepL.
- Translate text.

## Not Implemented

- Support for the [(beta) document translation endpoint](https://www.deepl.com/docs-api/translating-documents/).
- Support for the [XML handling flags](https://www.deepl.com/docs-api/translating-text/) in the translation endpoint.

## See Also

There are comparable implementations for [Python](https://github.com/mgruner/deepl-api-py) and [Ruby](https://github.com/mgruner/deepl-api-rb).

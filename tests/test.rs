extern crate dprint_development;
extern crate dprint_plugin_mago;

use std::path::PathBuf;
use std::sync::Arc;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_mago::configuration::Configuration;
use dprint_plugin_mago::configuration::resolve_config;
use dprint_plugin_mago::*;

#[test]
fn test_specs() {
  let global_config = GlobalConfiguration::default();

  run_specs(
    &PathBuf::from("./tests/specs"),
    &ParseSpecOptions {
      default_file_name: "file.php",
    },
    &RunSpecsOptions {
      fix_failures: std::env::var("FIX").is_ok(),
      format_twice: true,
    },
    {
      let global_config = global_config.clone();
      Arc::new(move |file_path, file_text, spec_config| {
        let spec_config: ConfigKeyMap = serde_json::from_value(spec_config.clone().into()).unwrap();
        let config_result = resolve_config(spec_config, &global_config);
        ensure_no_diagnostics(&config_result.diagnostics);

        format_text(file_path, file_text, &config_result.config).map_err(anyhow::Error::from)
      })
    },
    Arc::new(move |_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing.")),
  )
}

#[test]
fn should_fail_on_parse_error_php() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.php"), "<?php\nfunction test( {}", &config).unwrap_err();
  assert!(!err.to_string().is_empty());
}

#[test]
fn should_format_basic_php() {
  let config = Configuration::default();
  let result = format_text(&PathBuf::from("./file.php"), "<?php\necho   'hello'  ;", &config).unwrap();
  assert!(result.is_some());
}

#[test]
fn should_return_none_for_non_php_files() {
  let config = Configuration::default();
  let result = format_text(&PathBuf::from("./file.js"), "const x = 1;", &config).unwrap();
  assert!(result.is_none());
}

#[test]
fn should_format_range_in_statement() {
  let text = "<?php\n\n$a=1;\n$b   =   2;\n$c=3;\n";
  let start = text.find("=   2").unwrap();
  let result = format_php_range(text, start..start + 1);
  assert_eq!(result.unwrap(), "<?php\n\n$a=1;\n$b = 2;\n$c=3;\n");
}

#[test]
fn should_format_range_in_class_member() {
  let text =
    "<?php\n\nclass A\n{\n    public function a(){return 1;}\n\n    public function   b( $x ){return $x;}\n}\n";
  // in the method's signature, so the whole method is the smallest enclosing member
  let start = text.find("b( $x )").unwrap();
  let result = format_php_range(text, start..start + 7);
  assert_eq!(
    result.unwrap(),
    "<?php\n\nclass A\n{\n    public function a(){return 1;}\n\n    public function b($x)\n    {\n        return $x;\n    }\n}\n"
  );
}

#[test]
fn should_return_none_for_formatted_range() {
  let text = "<?php\n\n$a=1;\n$b = 2;\n$c=3;\n";
  let start = text.find("$b").unwrap();
  let result = format_php_range(text, start..start + 7);
  assert!(result.is_none());
}

#[test]
fn should_format_range_after_multi_byte_chars() {
  let text = "<?php\n\n$a='héllo 😀';\n$b   =   'wörld';\n$c=3;\n";
  let start = text.find("$b").unwrap();
  let end = text.find("$c").unwrap();
  let result = format_php_range(text, start..end);
  assert_eq!(result.unwrap(), "<?php\n\n$a='héllo 😀';\n$b = 'wörld';\n$c=3;\n");
}

#[test]
fn should_return_none_for_range_in_non_php_files() {
  let result = format_text_range(
    &PathBuf::from("./file.js"),
    "const x=1;",
    0..10,
    &Configuration::default(),
  )
  .unwrap();
  assert!(result.is_none());
}

fn format_php_range(text: &str, range: std::ops::Range<usize>) -> Option<String> {
  format_text_range(&PathBuf::from("./file.php"), text, range, &Configuration::default()).unwrap()
}

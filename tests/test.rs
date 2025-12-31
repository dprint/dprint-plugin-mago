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

        format_text(file_path, &file_text, &config_result.config)
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
  let result = format_text(
    &PathBuf::from("./file.php"),
    "<?php\necho   'hello'  ;",
    &config,
  )
  .unwrap();
  assert!(result.is_some());
}

#[test]
fn should_return_none_for_non_php_files() {
  let config = Configuration::default();
  let result = format_text(&PathBuf::from("./file.js"), "const x = 1;", &config).unwrap();
  assert!(result.is_none());
}

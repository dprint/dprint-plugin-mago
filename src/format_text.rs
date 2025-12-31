use std::borrow::Cow;
use std::path::Path;

use anyhow::Result;
use bumpalo::Bump;
use mago_formatter::settings::BraceStyle;
use mago_formatter::settings::EndOfLine;
use mago_formatter::settings::FormatSettings;
use mago_formatter::settings::MethodChainBreakingStyle;
use mago_formatter::settings::NullTypeHint;
use mago_formatter::Formatter;
use mago_php_version::PHPVersion;

use crate::configuration::Configuration;

pub fn format_text(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>> {
  let lower_ext = file_path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|s| s.to_lowercase());

  // Only handle PHP files
  if !matches!(lower_ext.as_deref(), Some("php")) {
    return Ok(None);
  }

  let arena = Bump::new();
  let php_version = PHPVersion::new(
    config.php_version_major.unwrap_or(8) as u32,
    config.php_version_minor.unwrap_or(4) as u32,
    0,
  );
  let settings = build_format_settings(config);
  let formatter = Formatter::new(&arena, php_version, settings);

  let file_name = file_path.to_string_lossy().into_owned();
  let formatted = formatter.format_code(Cow::Owned(file_name), Cow::Owned(input_text.to_string()))?;

  if formatted == input_text {
    Ok(None)
  } else {
    Ok(Some(formatted.to_string()))
  }
}

fn build_format_settings(config: &Configuration) -> FormatSettings {
  let mut settings = FormatSettings::default();

  // Core layout settings
  if let Some(print_width) = config.print_width {
    settings.print_width = print_width as usize;
  }
  if let Some(tab_width) = config.tab_width {
    settings.tab_width = tab_width as usize;
  }
  if let Some(use_tabs) = config.use_tabs {
    settings.use_tabs = use_tabs;
  }
  if let Some(ref end_of_line) = config.end_of_line {
    settings.end_of_line = match end_of_line {
      crate::configuration::EndOfLine::Lf => EndOfLine::Lf,
      crate::configuration::EndOfLine::Crlf => EndOfLine::Crlf,
      crate::configuration::EndOfLine::Cr => EndOfLine::Cr,
    };
  }

  // Quote and punctuation
  if let Some(single_quote) = config.single_quote {
    settings.single_quote = single_quote;
  }
  if let Some(trailing_comma) = config.trailing_comma {
    settings.trailing_comma = trailing_comma;
  }
  if let Some(remove_trailing_close_tag) = config.remove_trailing_close_tag {
    settings.remove_trailing_close_tag = remove_trailing_close_tag;
  }

  // Brace styles
  if let Some(ref style) = config.control_brace_style {
    settings.control_brace_style = match style {
      crate::configuration::BraceStyle::SameLine => BraceStyle::SameLine,
      crate::configuration::BraceStyle::NextLine => BraceStyle::NextLine,
    };
  }
  if let Some(ref style) = config.closure_brace_style {
    settings.closure_brace_style = match style {
      crate::configuration::BraceStyle::SameLine => BraceStyle::SameLine,
      crate::configuration::BraceStyle::NextLine => BraceStyle::NextLine,
    };
  }
  if let Some(ref style) = config.function_brace_style {
    settings.function_brace_style = match style {
      crate::configuration::BraceStyle::SameLine => BraceStyle::SameLine,
      crate::configuration::BraceStyle::NextLine => BraceStyle::NextLine,
    };
  }
  if let Some(ref style) = config.method_brace_style {
    settings.method_brace_style = match style {
      crate::configuration::BraceStyle::SameLine => BraceStyle::SameLine,
      crate::configuration::BraceStyle::NextLine => BraceStyle::NextLine,
    };
  }
  if let Some(ref style) = config.classlike_brace_style {
    settings.classlike_brace_style = match style {
      crate::configuration::BraceStyle::SameLine => BraceStyle::SameLine,
      crate::configuration::BraceStyle::NextLine => BraceStyle::NextLine,
    };
  }

  // Empty brace handling
  if let Some(v) = config.inline_empty_control_braces {
    settings.inline_empty_control_braces = v;
  }
  if let Some(v) = config.inline_empty_closure_braces {
    settings.inline_empty_closure_braces = v;
  }
  if let Some(v) = config.inline_empty_function_braces {
    settings.inline_empty_function_braces = v;
  }
  if let Some(v) = config.inline_empty_method_braces {
    settings.inline_empty_method_braces = v;
  }
  if let Some(v) = config.inline_empty_constructor_braces {
    settings.inline_empty_constructor_braces = v;
  }
  if let Some(v) = config.inline_empty_classlike_braces {
    settings.inline_empty_classlike_braces = v;
  }
  if let Some(v) = config.inline_empty_anonymous_class_braces {
    settings.inline_empty_anonymous_class_braces = v;
  }

  // Method chaining
  if let Some(ref style) = config.method_chain_breaking_style {
    settings.method_chain_breaking_style = match style {
      crate::configuration::MethodChainBreakingStyle::SameLine => MethodChainBreakingStyle::SameLine,
      crate::configuration::MethodChainBreakingStyle::NextLine => MethodChainBreakingStyle::NextLine,
    };
  }
  if let Some(v) = config.first_method_chain_on_new_line {
    settings.first_method_chain_on_new_line = v;
  }
  if let Some(v) = config.preserve_breaking_member_access_chain {
    settings.preserve_breaking_member_access_chain = v;
  }

  // Preservation flags
  if let Some(v) = config.preserve_breaking_argument_list {
    settings.preserve_breaking_argument_list = v;
  }
  if let Some(v) = config.preserve_breaking_array_like {
    settings.preserve_breaking_array_like = v;
  }
  if let Some(v) = config.preserve_breaking_parameter_list {
    settings.preserve_breaking_parameter_list = v;
  }
  if let Some(v) = config.preserve_breaking_attribute_list {
    settings.preserve_breaking_attribute_list = v;
  }
  if let Some(v) = config.preserve_breaking_conditional_expression {
    settings.preserve_breaking_conditional_expression = v;
  }

  // Operator and structural settings
  if let Some(v) = config.break_promoted_properties_list {
    settings.break_promoted_properties_list = v;
  }
  if let Some(v) = config.line_before_binary_operator {
    settings.line_before_binary_operator = v;
  }
  if let Some(v) = config.always_break_named_arguments_list {
    settings.always_break_named_arguments_list = v;
  }
  if let Some(v) = config.always_break_attribute_named_argument_lists {
    settings.always_break_attribute_named_argument_lists = v;
  }
  if let Some(v) = config.array_table_style_alignment {
    settings.array_table_style_alignment = v;
  }
  if let Some(v) = config.align_assignment_like {
    settings.align_assignment_like = v;
  }

  // Use statement organization
  if let Some(v) = config.sort_uses {
    settings.sort_uses = v;
  }
  if let Some(v) = config.sort_class_methods {
    settings.sort_class_methods = v;
  }
  if let Some(v) = config.separate_use_types {
    settings.separate_use_types = v;
  }
  if let Some(v) = config.expand_use_groups {
    settings.expand_use_groups = v;
  }

  // Type hints and syntax
  if let Some(ref style) = config.null_type_hint {
    settings.null_type_hint = match style {
      crate::configuration::NullTypeHint::Question => NullTypeHint::Question,
      crate::configuration::NullTypeHint::NullPipe => NullTypeHint::NullPipe,
    };
  }
  if let Some(v) = config.parentheses_around_new_in_member_access {
    settings.parentheses_around_new_in_member_access = v;
  }
  if let Some(v) = config.parentheses_in_new_expression {
    settings.parentheses_in_new_expression = v;
  }
  if let Some(v) = config.parentheses_in_exit_and_die {
    settings.parentheses_in_exit_and_die = v;
  }
  if let Some(v) = config.parentheses_in_attribute {
    settings.parentheses_in_attribute = v;
  }

  // Space control settings
  if let Some(v) = config.space_before_arrow_function_parameter_list_parenthesis {
    settings.space_before_arrow_function_parameter_list_parenthesis = v;
  }
  if let Some(v) = config.space_before_closure_parameter_list_parenthesis {
    settings.space_before_closure_parameter_list_parenthesis = v;
  }
  if let Some(v) = config.space_before_hook_parameter_list_parenthesis {
    settings.space_before_hook_parameter_list_parenthesis = v;
  }
  if let Some(v) = config.space_before_closure_use_clause_parenthesis {
    settings.space_before_closure_use_clause_parenthesis = v;
  }
  if let Some(v) = config.space_after_cast_unary_prefix_operators {
    settings.space_after_cast_unary_prefix_operators = v;
  }
  if let Some(v) = config.space_after_reference_unary_prefix_operator {
    settings.space_after_reference_unary_prefix_operator = v;
  }
  if let Some(v) = config.space_after_error_control_unary_prefix_operator {
    settings.space_after_error_control_unary_prefix_operator = v;
  }
  if let Some(v) = config.space_after_logical_not_unary_prefix_operator {
    settings.space_after_logical_not_unary_prefix_operator = v;
  }
  if let Some(v) = config.space_after_bitwise_not_unary_prefix_operator {
    settings.space_after_bitwise_not_unary_prefix_operator = v;
  }
  if let Some(v) = config.space_after_increment_unary_prefix_operator {
    settings.space_after_increment_unary_prefix_operator = v;
  }
  if let Some(v) = config.space_after_decrement_unary_prefix_operator {
    settings.space_after_decrement_unary_prefix_operator = v;
  }
  if let Some(v) = config.space_after_additive_unary_prefix_operator {
    settings.space_after_additive_unary_prefix_operator = v;
  }
  if let Some(v) = config.space_around_concatenation_binary_operator {
    settings.space_around_concatenation_binary_operator = v;
  }
  if let Some(v) = config.space_around_assignment_in_declare {
    settings.space_around_assignment_in_declare = v;
  }
  if let Some(v) = config.space_within_grouping_parenthesis {
    settings.space_within_grouping_parenthesis = v;
  }

  // Blank line configuration
  if let Some(v) = config.empty_line_after_control_structure {
    settings.empty_line_after_control_structure = v;
  }
  if let Some(v) = config.empty_line_after_opening_tag {
    settings.empty_line_after_opening_tag = v;
  }
  if let Some(v) = config.empty_line_after_declare {
    settings.empty_line_after_declare = v;
  }
  if let Some(v) = config.empty_line_after_namespace {
    settings.empty_line_after_namespace = v;
  }
  if let Some(v) = config.empty_line_after_use {
    settings.empty_line_after_use = v;
  }
  if let Some(v) = config.empty_line_after_symbols {
    settings.empty_line_after_symbols = v;
  }
  if let Some(v) = config.empty_line_between_same_symbols {
    settings.empty_line_between_same_symbols = v;
  }
  if let Some(v) = config.empty_line_after_class_like_constant {
    settings.empty_line_after_class_like_constant = v;
  }
  if let Some(v) = config.empty_line_after_enum_case {
    settings.empty_line_after_enum_case = v;
  }
  if let Some(v) = config.empty_line_after_trait_use {
    settings.empty_line_after_trait_use = v;
  }
  if let Some(v) = config.empty_line_after_property {
    settings.empty_line_after_property = v;
  }
  if let Some(v) = config.empty_line_after_method {
    settings.empty_line_after_method = v;
  }
  if let Some(v) = config.empty_line_before_return {
    settings.empty_line_before_return = v;
  }
  if let Some(v) = config.empty_line_before_dangling_comments {
    settings.empty_line_before_dangling_comments = v;
  }
  if let Some(v) = config.separate_class_like_members {
    settings.separate_class_like_members = v;
  }

  settings
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn formats_basic_php() {
    let input = "<?php\necho 'hello';\n";
    let config = crate::configuration::Configuration::default();
    let result = format_text(std::path::Path::new("test.php"), input, &config);
    assert!(result.is_ok());
  }

  #[test]
  fn returns_none_for_non_php() {
    let input = "const x = 1;";
    let config = crate::configuration::Configuration::default();
    let result = format_text(std::path::Path::new("test.js"), input, &config)
      .unwrap();
    assert!(result.is_none());
  }
}

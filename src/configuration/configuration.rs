use dprint_core::configuration::ParseConfigurationError;
use dprint_core::generate_str_to_from;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EndOfLine {
  Auto,
  Lf,
  Cr,
  Crlf,
}

generate_str_to_from![EndOfLine, [Auto, "auto"], [Lf, "lf"], [Cr, "cr"], [Crlf, "crlf"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BraceStyle {
  SameLine,
  NextLine,
  AlwaysNextLine,
}

generate_str_to_from![
  BraceStyle,
  [SameLine, "same-line"],
  [NextLine, "next-line"],
  [AlwaysNextLine, "always-next-line"]
];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortUses {
  Preserve,
  AlphanumericAscending,
  AlphanumericDescending,
  LengthAscending,
  LengthDescending,
}

impl std::str::FromStr for SortUses {
  type Err = ParseConfigurationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "false" | "preserve" => Ok(Self::Preserve),
      "true" | "alphanumeric-ascending" => Ok(Self::AlphanumericAscending),
      "alphanumeric-descending" => Ok(Self::AlphanumericDescending),
      "length-ascending" => Ok(Self::LengthAscending),
      "length-descending" => Ok(Self::LengthDescending),
      _ => Err(ParseConfigurationError(String::from(s))),
    }
  }
}

impl std::fmt::Display for SortUses {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let value = match self {
      Self::Preserve => "preserve",
      Self::AlphanumericAscending => "alphanumeric-ascending",
      Self::AlphanumericDescending => "alphanumeric-descending",
      Self::LengthAscending => "length-ascending",
      Self::LengthDescending => "length-descending",
    };

    write!(f, "{value}")
  }
}

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MethodChainBreakingStyle {
  SameLine,
  NextLine,
}

generate_str_to_from![
  MethodChainBreakingStyle,
  [SameLine, "same-line"],
  [NextLine, "next-line"]
];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NullTypeHint {
  Question,
  NullPipe,
  NullPipeLast,
}

generate_str_to_from![
  NullTypeHint,
  [Question, "question"],
  [NullPipe, "null-pipe"],
  [NullPipeLast, "null-pipe-last"]
];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortOrder {
  Preserve,
  AlphanumericAscending,
  AlphanumericDescending,
  LengthAscending,
  LengthDescending,
}

generate_str_to_from![
  SortOrder,
  [Preserve, "preserve"],
  [AlphanumericAscending, "alphanumeric-ascending"],
  [AlphanumericDescending, "alphanumeric-descending"],
  [LengthAscending, "length-ascending"],
  [LengthDescending, "length-descending"]
];

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
  // PHP version settings
  pub php_version_major: Option<u8>,
  pub php_version_minor: Option<u8>,

  // Core layout settings
  pub print_width: Option<u16>,
  pub tab_width: Option<u8>,
  pub use_tabs: Option<bool>,
  pub end_of_line: Option<EndOfLine>,

  // Quote and punctuation
  pub single_quote: Option<bool>,
  pub trailing_comma: Option<bool>,
  pub remove_trailing_close_tag: Option<bool>,

  // Brace styles
  pub control_brace_style: Option<BraceStyle>,
  pub following_clause_on_newline: Option<bool>,
  pub closure_brace_style: Option<BraceStyle>,
  pub function_brace_style: Option<BraceStyle>,
  pub method_brace_style: Option<BraceStyle>,
  pub classlike_brace_style: Option<BraceStyle>,

  // Empty brace handling
  pub inline_empty_control_braces: Option<bool>,
  pub inline_empty_closure_braces: Option<bool>,
  pub inline_empty_function_braces: Option<bool>,
  pub inline_empty_method_braces: Option<bool>,
  pub inline_empty_constructor_braces: Option<bool>,
  pub inline_empty_classlike_braces: Option<bool>,
  pub inline_empty_anonymous_class_braces: Option<bool>,

  // Method chaining
  pub method_chain_breaking_style: Option<MethodChainBreakingStyle>,
  pub first_method_chain_on_new_line: Option<bool>,
  pub method_chain_semicolon_on_next_line: Option<bool>,
  pub preserve_breaking_member_access_chain: Option<bool>,
  pub preserve_breaking_member_access_chain_first_method_on_same_line: Option<bool>,

  // Preservation flags
  pub preserve_breaking_argument_list: Option<bool>,
  pub inline_single_breaking_value_argument: Option<bool>,
  pub preserve_breaking_array_like: Option<bool>,
  pub preserve_breaking_parameter_list: Option<bool>,
  pub preserve_breaking_attribute_list: Option<bool>,
  pub preserve_breaking_conditional_expression: Option<bool>,
  pub preserve_breaking_condition_expression: Option<bool>,
  pub preserve_breaking_binary_expression: Option<bool>,

  // Operator and structural settings
  pub break_promoted_properties_list: Option<bool>,
  pub parameter_attribute_on_new_line: Option<bool>,
  pub line_before_binary_operator: Option<bool>,
  pub indent_binary_expression_continuation: Option<bool>,
  pub omit_redundant_arithmetic_binary_expression_parentheses: Option<bool>,
  pub omit_redundant_bitwise_binary_expression_parentheses: Option<bool>,
  pub preserve_redundant_logical_binary_expression_parentheses: Option<bool>,
  pub always_break_named_arguments_list: Option<bool>,
  pub always_break_attribute_named_argument_lists: Option<bool>,
  pub align_named_arguments: Option<bool>,
  pub align_parameters: Option<bool>,
  pub array_table_style_alignment: Option<bool>,
  pub align_assignment_like: Option<bool>,

  // Use statement organization
  pub sort_uses: Option<SortUses>,
  pub sort_class_methods: Option<bool>,
  pub separate_use_types: Option<bool>,
  pub expand_use_groups: Option<bool>,

  // Type hints and syntax
  pub null_type_hint: Option<NullTypeHint>,
  pub parentheses_around_new_in_member_access: Option<bool>,
  pub parentheses_in_new_expression: Option<bool>,
  pub parentheses_in_exit_and_die: Option<bool>,
  pub parentheses_in_attribute: Option<bool>,

  // Space control settings
  pub space_before_arrow_function_parameter_list_parenthesis: Option<bool>,
  pub space_before_closure_parameter_list_parenthesis: Option<bool>,
  pub space_before_hook_parameter_list_parenthesis: Option<bool>,
  pub inline_abstract_property_hooks: Option<bool>,
  pub space_before_closure_use_clause_parenthesis: Option<bool>,
  pub space_after_cast_unary_prefix_operators: Option<bool>,
  pub space_after_reference_unary_prefix_operator: Option<bool>,
  pub space_after_error_control_unary_prefix_operator: Option<bool>,
  pub space_after_logical_not_unary_prefix_operator: Option<bool>,
  pub space_after_bitwise_not_unary_prefix_operator: Option<bool>,
  pub space_after_increment_unary_prefix_operator: Option<bool>,
  pub space_after_decrement_unary_prefix_operator: Option<bool>,
  pub space_after_additive_unary_prefix_operator: Option<bool>,
  pub space_around_concatenation_binary_operator: Option<bool>,
  pub space_around_assignment_in_declare: Option<bool>,
  pub space_within_grouping_parenthesis: Option<bool>,

  // Blank line configuration
  pub empty_line_after_control_structure: Option<bool>,
  pub opening_tag_on_own_line: Option<bool>,
  pub empty_line_after_opening_tag: Option<bool>,
  pub empty_line_after_declare: Option<bool>,
  pub combine_opening_tag_and_declare: Option<bool>,
  pub empty_line_after_namespace: Option<bool>,
  pub empty_line_after_use: Option<bool>,
  pub empty_line_after_symbols: Option<bool>,
  pub empty_line_between_same_symbols: Option<bool>,
  pub empty_line_after_class_like_constant: Option<bool>,
  pub empty_line_after_class_like_open: Option<bool>,
  pub empty_line_before_class_like_close: Option<bool>,
  pub empty_line_after_enum_case: Option<bool>,
  pub empty_line_after_trait_use: Option<bool>,
  pub empty_line_after_property: Option<bool>,
  pub empty_line_after_method: Option<bool>,
  pub empty_line_before_return: Option<bool>,
  pub empty_line_before_dangling_comments: Option<bool>,
  pub separate_class_like_members: Option<bool>,
  pub attributes_order: Option<SortOrder>,
  pub separate_attributes: Option<bool>,
  pub separate_trait_use: Option<bool>,
  pub indent_heredoc: Option<bool>,
  pub uppercase_literal_keyword: Option<bool>,
}

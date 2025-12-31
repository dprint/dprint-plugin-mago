use super::Configuration;
use super::EndOfLine;
use dprint_core::configuration::*;

/// Resolves configuration from a collection of key value strings.
///
/// # Example
///
/// ```
/// use dprint_core::configuration::ConfigKeyMap;
/// use dprint_core::configuration::resolve_global_config;
/// use dprint_plugin_mago::configuration::resolve_config;
///
/// let mut config_map = ConfigKeyMap::new(); // get a collection of key value pairs from somewhere
/// let global_config_result = resolve_global_config(&mut config_map);
///
/// // check global_config_result.diagnostics here...
///
/// let config_result = resolve_config(
///     config_map,
///     &global_config_result.config
/// );
///
/// // check config_result.diagnostics here and use config_result.config
/// ```
pub fn resolve_config(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
  let mut diagnostics = Vec::new();
  let mut config = config;

  // Get global values that can be used as fallbacks
  let use_tabs = get_nullable_value(&mut config, "useTabs", &mut diagnostics).or(global_config.use_tabs);
  let tab_width = get_nullable_value(&mut config, "tabWidth", &mut diagnostics)
    .or_else(|| get_nullable_value(&mut config, "indentWidth", &mut diagnostics))
    .or(global_config.indent_width);
  let print_width = get_nullable_value(&mut config, "printWidth", &mut diagnostics)
    .or_else(|| get_nullable_value(&mut config, "lineWidth", &mut diagnostics))
    .or(global_config.line_width.map(|l| std::cmp::min(u16::MAX as u32, l) as u16));

  let resolved_config = Configuration {
    // PHP version settings
    php_version_major: get_nullable_value(&mut config, "phpVersionMajor", &mut diagnostics),
    php_version_minor: get_nullable_value(&mut config, "phpVersionMinor", &mut diagnostics),

    // Core layout settings
    print_width,
    tab_width: tab_width.map(|v| v as u8),
    use_tabs,
    end_of_line: get_nullable_value(&mut config, "endOfLine", &mut diagnostics).or(
      match global_config.new_line_kind {
        Some(NewLineKind::CarriageReturnLineFeed) => Some(EndOfLine::Crlf),
        Some(NewLineKind::LineFeed) => Some(EndOfLine::Lf),
        _ => None,
      },
    ),

    // Quote and punctuation
    single_quote: get_nullable_value(&mut config, "singleQuote", &mut diagnostics),
    trailing_comma: get_nullable_value(&mut config, "trailingComma", &mut diagnostics),
    remove_trailing_close_tag: get_nullable_value(&mut config, "removeTrailingCloseTag", &mut diagnostics),

    // Brace styles
    control_brace_style: get_nullable_value(&mut config, "controlBraceStyle", &mut diagnostics),
    closure_brace_style: get_nullable_value(&mut config, "closureBraceStyle", &mut diagnostics),
    function_brace_style: get_nullable_value(&mut config, "functionBraceStyle", &mut diagnostics),
    method_brace_style: get_nullable_value(&mut config, "methodBraceStyle", &mut diagnostics),
    classlike_brace_style: get_nullable_value(&mut config, "classlikeBraceStyle", &mut diagnostics),

    // Empty brace handling
    inline_empty_control_braces: get_nullable_value(&mut config, "inlineEmptyControlBraces", &mut diagnostics),
    inline_empty_closure_braces: get_nullable_value(&mut config, "inlineEmptyClosureBraces", &mut diagnostics),
    inline_empty_function_braces: get_nullable_value(&mut config, "inlineEmptyFunctionBraces", &mut diagnostics),
    inline_empty_method_braces: get_nullable_value(&mut config, "inlineEmptyMethodBraces", &mut diagnostics),
    inline_empty_constructor_braces: get_nullable_value(&mut config, "inlineEmptyConstructorBraces", &mut diagnostics),
    inline_empty_classlike_braces: get_nullable_value(&mut config, "inlineEmptyClasslikeBraces", &mut diagnostics),
    inline_empty_anonymous_class_braces: get_nullable_value(
      &mut config,
      "inlineEmptyAnonymousClassBraces",
      &mut diagnostics,
    ),

    // Method chaining
    method_chain_breaking_style: get_nullable_value(&mut config, "methodChainBreakingStyle", &mut diagnostics),
    first_method_chain_on_new_line: get_nullable_value(&mut config, "firstMethodChainOnNewLine", &mut diagnostics),
    preserve_breaking_member_access_chain: get_nullable_value(
      &mut config,
      "preserveBreakingMemberAccessChain",
      &mut diagnostics,
    ),

    // Preservation flags
    preserve_breaking_argument_list: get_nullable_value(&mut config, "preserveBreakingArgumentList", &mut diagnostics),
    preserve_breaking_array_like: get_nullable_value(&mut config, "preserveBreakingArrayLike", &mut diagnostics),
    preserve_breaking_parameter_list: get_nullable_value(
      &mut config,
      "preserveBreakingParameterList",
      &mut diagnostics,
    ),
    preserve_breaking_attribute_list: get_nullable_value(
      &mut config,
      "preserveBreakingAttributeList",
      &mut diagnostics,
    ),
    preserve_breaking_conditional_expression: get_nullable_value(
      &mut config,
      "preserveBreakingConditionalExpression",
      &mut diagnostics,
    ),

    // Operator and structural settings
    break_promoted_properties_list: get_nullable_value(&mut config, "breakPromotedPropertiesList", &mut diagnostics),
    line_before_binary_operator: get_nullable_value(&mut config, "lineBeforeBinaryOperator", &mut diagnostics),
    always_break_named_arguments_list: get_nullable_value(
      &mut config,
      "alwaysBreakNamedArgumentsList",
      &mut diagnostics,
    ),
    always_break_attribute_named_argument_lists: get_nullable_value(
      &mut config,
      "alwaysBreakAttributeNamedArgumentLists",
      &mut diagnostics,
    ),
    array_table_style_alignment: get_nullable_value(&mut config, "arrayTableStyleAlignment", &mut diagnostics),
    align_assignment_like: get_nullable_value(&mut config, "alignAssignmentLike", &mut diagnostics),

    // Use statement organization
    sort_uses: get_nullable_value(&mut config, "sortUses", &mut diagnostics),
    sort_class_methods: get_nullable_value(&mut config, "sortClassMethods", &mut diagnostics),
    separate_use_types: get_nullable_value(&mut config, "separateUseTypes", &mut diagnostics),
    expand_use_groups: get_nullable_value(&mut config, "expandUseGroups", &mut diagnostics),

    // Type hints and syntax
    null_type_hint: get_nullable_value(&mut config, "nullTypeHint", &mut diagnostics),
    parentheses_around_new_in_member_access: get_nullable_value(
      &mut config,
      "parenthesesAroundNewInMemberAccess",
      &mut diagnostics,
    ),
    parentheses_in_new_expression: get_nullable_value(&mut config, "parenthesesInNewExpression", &mut diagnostics),
    parentheses_in_exit_and_die: get_nullable_value(&mut config, "parenthesesInExitAndDie", &mut diagnostics),
    parentheses_in_attribute: get_nullable_value(&mut config, "parenthesesInAttribute", &mut diagnostics),

    // Space control settings
    space_before_arrow_function_parameter_list_parenthesis: get_nullable_value(
      &mut config,
      "spaceBeforeArrowFunctionParameterListParenthesis",
      &mut diagnostics,
    ),
    space_before_closure_parameter_list_parenthesis: get_nullable_value(
      &mut config,
      "spaceBeforeClosureParameterListParenthesis",
      &mut diagnostics,
    ),
    space_before_hook_parameter_list_parenthesis: get_nullable_value(
      &mut config,
      "spaceBeforeHookParameterListParenthesis",
      &mut diagnostics,
    ),
    space_before_closure_use_clause_parenthesis: get_nullable_value(
      &mut config,
      "spaceBeforeClosureUseClauseParenthesis",
      &mut diagnostics,
    ),
    space_after_cast_unary_prefix_operators: get_nullable_value(
      &mut config,
      "spaceAfterCastUnaryPrefixOperators",
      &mut diagnostics,
    ),
    space_after_reference_unary_prefix_operator: get_nullable_value(
      &mut config,
      "spaceAfterReferenceUnaryPrefixOperator",
      &mut diagnostics,
    ),
    space_after_error_control_unary_prefix_operator: get_nullable_value(
      &mut config,
      "spaceAfterErrorControlUnaryPrefixOperator",
      &mut diagnostics,
    ),
    space_after_logical_not_unary_prefix_operator: get_nullable_value(
      &mut config,
      "spaceAfterLogicalNotUnaryPrefixOperator",
      &mut diagnostics,
    ),
    space_after_bitwise_not_unary_prefix_operator: get_nullable_value(
      &mut config,
      "spaceAfterBitwiseNotUnaryPrefixOperator",
      &mut diagnostics,
    ),
    space_after_increment_unary_prefix_operator: get_nullable_value(
      &mut config,
      "spaceAfterIncrementUnaryPrefixOperator",
      &mut diagnostics,
    ),
    space_after_decrement_unary_prefix_operator: get_nullable_value(
      &mut config,
      "spaceAfterDecrementUnaryPrefixOperator",
      &mut diagnostics,
    ),
    space_after_additive_unary_prefix_operator: get_nullable_value(
      &mut config,
      "spaceAfterAdditiveUnaryPrefixOperator",
      &mut diagnostics,
    ),
    space_around_concatenation_binary_operator: get_nullable_value(
      &mut config,
      "spaceAroundConcatenationBinaryOperator",
      &mut diagnostics,
    ),
    space_around_assignment_in_declare: get_nullable_value(
      &mut config,
      "spaceAroundAssignmentInDeclare",
      &mut diagnostics,
    ),
    space_within_grouping_parenthesis: get_nullable_value(
      &mut config,
      "spaceWithinGroupingParenthesis",
      &mut diagnostics,
    ),

    // Blank line configuration
    empty_line_after_control_structure: get_nullable_value(
      &mut config,
      "emptyLineAfterControlStructure",
      &mut diagnostics,
    ),
    empty_line_after_opening_tag: get_nullable_value(&mut config, "emptyLineAfterOpeningTag", &mut diagnostics),
    empty_line_after_declare: get_nullable_value(&mut config, "emptyLineAfterDeclare", &mut diagnostics),
    empty_line_after_namespace: get_nullable_value(&mut config, "emptyLineAfterNamespace", &mut diagnostics),
    empty_line_after_use: get_nullable_value(&mut config, "emptyLineAfterUse", &mut diagnostics),
    empty_line_after_symbols: get_nullable_value(&mut config, "emptyLineAfterSymbols", &mut diagnostics),
    empty_line_between_same_symbols: get_nullable_value(&mut config, "emptyLineBetweenSameSymbols", &mut diagnostics),
    empty_line_after_class_like_constant: get_nullable_value(
      &mut config,
      "emptyLineAfterClassLikeConstant",
      &mut diagnostics,
    ),
    empty_line_after_enum_case: get_nullable_value(&mut config, "emptyLineAfterEnumCase", &mut diagnostics),
    empty_line_after_trait_use: get_nullable_value(&mut config, "emptyLineAfterTraitUse", &mut diagnostics),
    empty_line_after_property: get_nullable_value(&mut config, "emptyLineAfterProperty", &mut diagnostics),
    empty_line_after_method: get_nullable_value(&mut config, "emptyLineAfterMethod", &mut diagnostics),
    empty_line_before_return: get_nullable_value(&mut config, "emptyLineBeforeReturn", &mut diagnostics),
    empty_line_before_dangling_comments: get_nullable_value(
      &mut config,
      "emptyLineBeforeDanglingComments",
      &mut diagnostics,
    ),
    separate_class_like_members: get_nullable_value(&mut config, "separateClassLikeMembers", &mut diagnostics),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

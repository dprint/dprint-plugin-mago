# dprint-plugin-mago

[![CI](https://github.com/dprint/dprint-plugin-mago/workflows/CI/badge.svg)](https://github.com/dprint/dprint-plugin-mago/actions?query=workflow%3ACI)

Adapter for [Mago](https://github.com/carthage-software/mago) for use as a formatting plugin in [dprint](https://github.com/dprint/dprint).

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

```shellsession
dprint add mago
# or install from npm
dprint add npm:@dprint/mago
```

Note: You do not need Mago installed globally as dprint will run Mago from the .wasm file in a sandboxed environment.

## Configuration

To add configuration, specify a `"mago"` key in your dprint.json:

```jsonc
{
  "mago": {
    "printWidth": 100,
    "useTabs": true,
  },
  "plugins": [
    // ...etc...
  ],
}
```

For an overview of the config, see https://dprint.dev/plugins/mago/config/

Note: The plugin does not understand Mago's configuration file because it runs sandboxed in a Wasm runtime—it has no access to the file system in order to read Mago's config.

### Formatter options

The plugin supports every Mago formatter setting. The configuration schema linked above provides defaults and value details; these are the available options, grouped by purpose:

- PHP version and layout: `phpVersionMajor`, `phpVersionMinor`, `printWidth`, `tabWidth`, `useTabs`, `endOfLine`, `singleQuote`, `trailingComma`, and `removeTrailingCloseTag`.
- Braces: `controlBraceStyle`, `followingClauseOnNewline`, `closureBraceStyle`, `functionBraceStyle`, `methodBraceStyle`, `classlikeBraceStyle`, `inlineEmptyControlBraces`, `inlineEmptyClosureBraces`, `inlineEmptyFunctionBraces`, `inlineEmptyMethodBraces`, `inlineEmptyConstructorBraces`, `inlineEmptyClasslikeBraces`, and `inlineEmptyAnonymousClassBraces`.
- Method chains and preservation: `methodChainBreakingStyle`, `firstMethodChainOnNewLine`, `methodChainSemicolonOnNextLine`, `preserveBreakingMemberAccessChain`, `preserveBreakingMemberAccessChainFirstMethodOnSameLine`, `preserveBreakingArgumentList`, `inlineSingleBreakingValueArgument`, `preserveBreakingArrayLike`, `preserveBreakingParameterList`, `preserveBreakingAttributeList`, `preserveBreakingConditionalExpression`, `preserveBreakingConditionExpression`, and `preserveBreakingBinaryExpression`.
- Expressions and alignment: `breakPromotedPropertiesList`, `parameterAttributeOnNewLine`, `lineBeforeBinaryOperator`, `indentBinaryExpressionContinuation`, `omitRedundantArithmeticBinaryExpressionParentheses`, `omitRedundantBitwiseBinaryExpressionParentheses`, `preserveRedundantLogicalBinaryExpressionParentheses`, `alwaysBreakNamedArgumentsList`, `alwaysBreakAttributeNamedArgumentLists`, `alignNamedArguments`, `alignParameters`, `arrayTableStyleAlignment`, and `alignAssignmentLike`.
- Imports and types: `sortUses`, `sortClassMethods`, `separateUseTypes`, `expandUseGroups`, `nullTypeHint`, `parenthesesAroundNewInMemberAccess`, `parenthesesInNewExpression`, `parenthesesInExitAndDie`, and `parenthesesInAttribute`.
- Spacing: `spaceBeforeArrowFunctionParameterListParenthesis`, `spaceBeforeClosureParameterListParenthesis`, `spaceBeforeHookParameterListParenthesis`, `inlineAbstractPropertyHooks`, `spaceBeforeClosureUseClauseParenthesis`, `spaceAfterCastUnaryPrefixOperators`, `spaceAfterReferenceUnaryPrefixOperator`, `spaceAfterErrorControlUnaryPrefixOperator`, `spaceAfterLogicalNotUnaryPrefixOperator`, `spaceAfterBitwiseNotUnaryPrefixOperator`, `spaceAfterIncrementUnaryPrefixOperator`, `spaceAfterDecrementUnaryPrefixOperator`, `spaceAfterAdditiveUnaryPrefixOperator`, `spaceAroundConcatenationBinaryOperator`, `spaceAroundAssignmentInDeclare`, and `spaceWithinGroupingParenthesis`.
- Blank lines and declarations: `emptyLineAfterControlStructure`, `openingTagOnOwnLine`, `emptyLineAfterOpeningTag`, `emptyLineAfterDeclare`, `combineOpeningTagAndDeclare`, `emptyLineAfterNamespace`, `emptyLineAfterUse`, `emptyLineAfterSymbols`, `emptyLineBetweenSameSymbols`, `emptyLineAfterClassLikeConstant`, `emptyLineAfterClassLikeOpen`, `emptyLineBeforeClassLikeClose`, `emptyLineAfterEnumCase`, `emptyLineAfterTraitUse`, `emptyLineAfterProperty`, `emptyLineAfterMethod`, `emptyLineBeforeReturn`, `emptyLineBeforeDanglingComments`, `separateClassLikeMembers`, `attributesOrder`, `separateAttributes`, `separateTraitUse`, `indentHeredoc`, and `uppercaseLiteralKeyword`.

`endOfLine` also accepts `auto`; brace styles also accept `always-next-line`; `nullTypeHint` also accepts `null-pipe-last`. `sortUses` and `attributesOrder` accept `preserve`, `alphanumeric-ascending`, `alphanumeric-descending`, `length-ascending`, and `length-descending`.

## JS Formatting API

- [JS Formatter](https://github.com/dprint/js-formatter) - Browser/Deno and Node
- [npm package](https://www.npmjs.com/package/@dprint/mago)

## Versioning

This repo automatically upgrades to the latest version of Mago once a day. You can check which version of Mago is being used by looking at the `mago-formatter` entry in the Cargo.toml file in this repo:

https://github.com/dprint/dprint-plugin-mago/blob/main/Cargo.toml

At the moment, the version of this plugin does not reflect the version of Mago. This is just in case there are any small bug fixes that need to be made as this plugin is quite new. After a while I'll try to match the versions.

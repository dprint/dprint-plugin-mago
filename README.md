# dprint-plugin-mago

[![CI](https://github.com/dprint/dprint-plugin-mago/workflows/CI/badge.svg)](https://github.com/dprint/dprint-plugin-mago/actions?query=workflow%3ACI)

Adapter for [Mago](https://github.com/carthage-software/mago) for use as a formatting plugin in [dprint](https://github.com/dprint/dprint).

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

```shellsession
dprint config add mago
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

## JS Formatting API

- [JS Formatter](https://github.com/dprint/js-formatter) - Browser/Deno and Node
- [npm package](https://www.npmjs.com/package/@dprint/mago)

## Versioning

This repo automatically upgrades to the latest version of Mago once a day. You can check which version of Mago is being used by looking at the `mago-formatter` entry in the Cargo.toml file in this repo:

https://github.com/dprint/dprint-plugin-mago/blob/main/Cargo.toml

At the moment, the version of this plugin does not reflect the version of Mago. This is just in case there are any small bug fixes that need to be made as this plugin is quite new. After a while I'll try to match the versions.

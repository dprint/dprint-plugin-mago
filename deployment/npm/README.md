# @dprint/mago

npm distribution of [dprint-plugin-mago](https://github.com/dprint/dprint-plugin-mago) which is an adapter plugin for [Mago](https://github.com/carthage-software/mago).

Use this with [@dprint/formatter](https://github.com/dprint/js-formatter) or just use @dprint/formatter and download the [dprint-plugin-mago Wasm file](https://github.com/dprint/dprint-plugin-mago/releases).

## Example

```ts
import { createFromBuffer } from "@dprint/formatter";
import { getBuffer } from "@dprint/mago";

const formatter = createFromBuffer(getBuffer());

console.log(
  formatter.formatText("test.php", "<?php\necho   '1'"),
);
```

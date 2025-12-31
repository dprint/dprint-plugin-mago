// @ts-check
const assert = require("assert");
const createFromBuffer = require("@dprint/formatter").createFromBuffer;
const getBuffer = require("./index").getBuffer;

const formatter = createFromBuffer(getBuffer());
const result = formatter.formatText({
  filePath: "file.php",
  fileText: "<?php\necho   5 ;",
});

assert.strictEqual(result, "<?php\n\necho 5;\n");

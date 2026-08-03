#!/usr/bin/env node

const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");

const source = fs.readFileSync("src/implementation/app/shell_script.js", "utf8");
const start = source.indexOf("// MARKHOLA_TABLE_KEYBOARD_START");
const end = source.indexOf("// MARKHOLA_TABLE_KEYBOARD_END");
assert.notEqual(start, -1, "keyboard helper start marker must exist");
assert.notEqual(end, -1, "keyboard helper end marker must exist");

class Element {
  constructor(classes = []) {
    this.classList = { contains: (name) => classes.includes(name) };
    this.clientWidth = 100;
    this.scrollWidth = 500;
    this.scrollLeft = 0;
  }
}

const document = { activeElement: null };
const context = { HTMLElement: Element, document };
vm.createContext(context);
vm.runInContext(`${source.slice(start, end)}\nthis.handle = handleTableRegionArrowKey;`, context);

const region = new Element(["markdown-table-region"]);
document.activeElement = region;
const event = (target, key) => ({
  target,
  key,
  prevented: false,
  preventDefault() { this.prevented = true; },
});

let key = event(region, "ArrowRight");
assert.equal(context.handle(key), true);
assert.equal(key.prevented, true);
assert.equal(region.scrollLeft, 80);

region.scrollLeft = 390;
context.handle(event(region, "ArrowRight"));
assert.equal(region.scrollLeft, 400, "right movement clamps at maximum");
region.scrollLeft = 10;
context.handle(event(region, "ArrowLeft"));
assert.equal(region.scrollLeft, 0, "left movement clamps at zero");

const link = new Element([]);
document.activeElement = region;
key = event(link, "ArrowRight");
assert.equal(context.handle(key), false, "interactive descendants retain native behavior");
assert.equal(key.prevented, false);
key = event(region, "ArrowDown");
assert.equal(context.handle(key), false, "vertical arrows retain document behavior");
assert.equal(key.prevented, false);

console.log("wide-table keyboard behavior tests passed");

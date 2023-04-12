# Bull Script
**Bull Script** is a bad project.
The file extension is ".bs"

Here is some example code.

```js
import std
proc main() then
    std.printval("Hello, world!\n")
end
```

## Setup

1. Install cargo
2. Fork or `git clone` this repo into a folder.
3. Use `cargo run -- ./test.bs` to run your file

## Language features

### "Procedures"
Define ~~function~~ procedures with parameters.
```
proc sayHello() then
    std.printlnval("Hello!")
end
```
### Variables
```
let x = 0
x = x + 1
```
### Namespaces
```
namespace tests then
    proc test1() then
        std.printlnval(1 is 1)
    end
end

tests.tests1()
```
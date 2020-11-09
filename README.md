# A simple programming language implemented by Rust
## Reference resources:
### Book: "Writing an Interpreter in Go" and "Writing a Compiler in Go"
### Other's impl: https://github.com/shuhei/cymbal

## How to use
### compile mode(default)
### current mode is 10% slower than Python
`cargo run --release`
### repl mode
`cargo run --release --eval`
### benchmark
`cargo run --release -- --benchmark`
### run file
`cargo run --release -- [file_name]`
## Syntax
### 1.Declare and Assign
```javascript
let a = 1;
a = 2;

let s = "hello";

let b = true;
```
### 2.Loop
```javascript
// the ';' is optional
let sum = 0;

for (let i = 0; i < 10; i = i + 1) {
    sum = sum + i
}
```
### 3.If/Else
```javascript
let a = 1

if a < 2 {
    a = 1
} else {
    a = 3
}
```
### 4.Function
```javascript
fn add(x, y) {
    // return is optional
    return x + y
}
add(1, 2) // 3

fn fibonacci(n) {
    //bracket is optional
    if (n < 2) {
        return n
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2)
    }
}
// below is the same
let add = fn(x, y) {
    x + y
}


let fibonacci = fn(n) {
    if n < 2 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```
### 5.Array and Directory
```javascript
let arr = [1, "3", 4 + 5]
let c = arr[2] // c = 9
arr[0] = 16 // arr = [16, "3", 9]
arr[4] // null
arr[4] = 10 // arr = [16, "3", 9, 10]


let map = {"a": 1, "b": 4, "c": 12}
let r = map["b"] // r = 4
map["a"] = 10 // map = {"a": 10, "b": 4, "c": 12}
map["bang"] // null
map["new"] = "I'm new" // map = {"a": 1, "b": 4, "c": 12, "new": "I'm new"}

```
### 6.Arithmetic operations
```javascript
let a = 1 + 1
let b = 1 - 1
let c = 1 * 1
let d = 1 / 1

let e = a + b
let f = a - c

let g = true
let h = false
let i = true == false // i = false
let j = 1 < 2 // j = true
let k = 1 >= 2 // k = false
```

### 7.Builtin Function
```javascript
let str = "hello"
let arr = [1, 4, 7]
let map = {"a": 1, "b": true, "c": "hey"}

// len(string | array)
len(str) // 5
len(arr) // 3

// first(string | array)
first(str) // "h"
first(arr) // 1

// last(string | array)
last(str) // "o"
last(arr) // 7

// rest(string | array)
rest(str) // "ello"
rest(arr) // [4, 7]

// push(string | array, any)
// not change the origin object
push(str, "world") // "hello world"
push(arr, 10) // [1, 4, 7, 10]

// print(any, any, ...)
print(str, "world") // "hello"\n"world"
print(arr) // [1, 4, 7]

// time()
time() // milliseconds since `1970-01-01 00:00:00 UTC`
```
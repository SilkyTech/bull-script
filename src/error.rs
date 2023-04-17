use std::process;

use crate::interpreter::Stack;

pub fn error_at(filen: &String, linen: &i32, charn: &i32, reason: &String) -> ! {
    panic!(
        "\x1b[31;1m[ERROR]\x1b[0m: At {}:{}:{}:\n\t{}",
        filen, linen, charn, reason
    );
}

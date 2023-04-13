use std::process;

use crate::interpreter::Location;

pub fn error_at(filen: &String, linen: &i32, charn: &i32, reason: &String) -> ! {
    panic!(
        "\x1b[31;1m[ERROR]\x1b[0m: At {}:{}:{}:\n\t{}",
        filen, linen, charn, reason
    );
}
pub fn error_at_with_stack_trace(
    stack: Vec<Location>,
    filen: &String,
    linen: &i32,
    charn: &i32,
    reason: &String,
) -> ! {
    let mut stack_formatted = "".to_string();
    for loc in stack {
        stack_formatted += format!(
            "In {} | {}:{}:{}",
            loc.name.join("."),
            loc.filen,
            loc.linen,
            loc.charn
        )
        .as_str();
    }
    eprintln!(
        "---------\n\x1b[31;1m[ERROR]\x1b[0m: At {}:{}:{}:\n\t{}\n====\nStack trace:\n{}\nTODO\n---------",
        filen, linen, charn, reason, stack_formatted
    );
    process::exit(1);
}
